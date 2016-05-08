use image;

use std::{ fs, io, mem, path, slice };
use std::io::Read;
use std::io::Write;

#[derive(Default)]
struct Le16 {
	buf: [u8; 2]
}

impl From<Le16> for u16 {
	fn from(le16: Le16) -> Self {
		u16::from_le(unsafe { mem::transmute(le16.buf) })
	}
}

impl From<u16> for Le16 {
	fn from(n: u16) -> Self {
		Le16 {
			buf: unsafe { mem::transmute(n.to_le()) }
		}
	}
}

#[derive(Default)]
#[repr(C, packed)]
struct TgaHeader {
	id_length: u8,
	color_map_type: u8,
	image_type: u8,

	// color map specification
	color_map_origin: Le16,
	color_map_length: Le16,
	color_map_depth: u8,

	// image specification
	x_origin: Le16,
	y_origin: Le16,
	width: Le16,
	height: Le16,
	bits_per_pixel: u8,
	image_descriptor: u8,
}

impl TgaHeader {
	fn as_u8_slice(&self) -> &[u8] {
		unsafe {
			slice::from_raw_parts(self as *const Self as *const u8, 
					      mem::size_of::<Self>())
		}
	}

	fn as_u8_slice_mut(&mut self) -> &mut [u8] {
		unsafe {
			slice::from_raw_parts_mut(self as *mut Self as *mut u8, 
						  mem::size_of::<Self>())
		}
	}
}

const TGA_FOOTER: &'static [u8] = b"TRUEVISION-XFILE.\0";

pub fn read(path: &path::Path) -> io::Result<image::Image> {
	let mut file = io::BufReader::new(try!(fs::File::open(path)));
	let mut header = TgaHeader::default();
	try!(file.read_exact(header.as_u8_slice_mut()));
	let width = u16::from(header.width) as usize;
	let height = u16::from(header.height) as usize;
	let (format, rle) = match (header.image_type, header.bits_per_pixel) {
		(2, 24) => (image::Format::Rgb, false),
		(2, 32) => (image::Format::Rgba, false),
		(3, 8) => (image::Format::Gray, false),
		(10, 24) => (image::Format::Rgb, true),
		(10, 32) => (image::Format::Rgba, true),
		(11, 8) => (image::Format::Gray, true),
		_ => return Err(io::Error::new(io::ErrorKind::Other, "invalid image_type")),
	};

	let nbytes = width * height * format.bytes_per_pixel();
	// FIXME: avoid zero init of data
	let mut data = vec![0; nbytes];
	if rle {
		try!(read_rle(&mut file, &mut data, format));
	} else {
		try!(file.read_exact(&mut data));
	}

	let mut image = image::Image::from_data(width, height, format, data);
	if header.image_descriptor & 0x10 != 0 {
		image.flip_horizontally();
	}
	if header.image_descriptor & 0x20 != 0{
		image.flip_vertically();
	}
	Ok(image)
}

fn read_rle(file: &mut io::Read, data: &mut Vec<u8>, format: image::Format) -> io::Result<()> {
	let bytes_per_pixel = format.bytes_per_pixel();
	let mut color = vec![0; bytes_per_pixel];
	let num_pixels = data.len() / bytes_per_pixel;
	let mut start_pixel = 0;
	while start_pixel < num_pixels {
		let mut code = [0; 1];
		try!(file.read_exact(&mut code));
		let run_length = (code[0] & !0x80) as usize + 1;
		let next_pixel = start_pixel + run_length;
		if next_pixel > num_pixels {
			return Err(io::Error::new(io::ErrorKind::Other, "invalid rle length"));
		}
		if code[0] & 0x80 == 0 {
			try!(file.read_exact(&mut data[start_pixel * bytes_per_pixel..][..run_length * bytes_per_pixel]));
			start_pixel = next_pixel;
		} else {
			// FIXME: read directly into data
			try!(file.read_exact(&mut color));
			while start_pixel < next_pixel {
				data[start_pixel * bytes_per_pixel..][..bytes_per_pixel].clone_from_slice(&color);
				start_pixel += 1;
			}
		}
	}
	Ok(())
}

pub fn write(image: &image::Image, path: &path::Path, rle: bool) -> io::Result<()> {
	let mut file = io::BufWriter::new(try!(fs::File::create(path)));
	let format = image.get_format();
	let width = image.get_width();
	let height = image.get_width();
	let image_type = match (format, rle) {
		(image::Format::Rgb, false) => 2,
		(image::Format::Rgba, false) => 2,
		(image::Format::Gray, false) => 3,
		(image::Format::Rgb, true) => 10,
		(image::Format::Rgba, true) => 10,
		(image::Format::Gray, true) => 11,
	};

	let header = TgaHeader {
		image_type: image_type,
		width: From::from(width as u16),
		height: From::from(height as u16),
		bits_per_pixel: format.bytes_per_pixel() as u8 * 8,
		image_descriptor: format.alpha_bytes_per_pixel() as u8 * 8,
		.. Default::default()
	};
	try!(file.write_all(header.as_u8_slice()));

	if rle {
		try!(write_rle(&mut file, image.get_data(), format));
	} else {
		try!(file.write_all(&image.get_data()));
	}

	try!(file.write_all(&[0; 4])); // developer area offset
	try!(file.write_all(&[0; 4])); // extension area offset
	try!(file.write_all(TGA_FOOTER));
	Ok(())
}

fn write_rle(file: &mut io::Write, data: &Vec<u8>, format: image::Format) -> io::Result<()> {
	let bytes_per_pixel = format.bytes_per_pixel();
	let num_pixels = data.len() / bytes_per_pixel;
	let mut start_pixel = 0;
	while start_pixel < num_pixels {
		let mut cur_color = &data[start_pixel * bytes_per_pixel..][..bytes_per_pixel];
		let mut raw = true;
		let mut run_length = 1;
		let mut next_pixel = start_pixel + 1;
		while next_pixel < num_pixels && run_length < 128 {
			let next_color = &data[next_pixel * bytes_per_pixel..][..bytes_per_pixel];
			let next_eq = cur_color == next_color;
			if run_length == 1 {
				raw = !next_eq;
			} else if raw && next_eq {
				// FIXME: only break if the next run_length > 2
				run_length -= 1;
				next_pixel -= 1;
				break;
			} else if !raw && !next_eq {
				break;
			}
			run_length += 1;
			next_pixel += 1;
			cur_color = next_color;
		}
		if raw {
			try!(file.write_all(&[(run_length-1) as u8]));
			try!(file.write_all(&data[start_pixel * bytes_per_pixel..][..run_length * bytes_per_pixel]));
		} else {
			try!(file.write_all(&[(run_length+127) as u8]));
			try!(file.write_all(cur_color));
		}
		start_pixel = next_pixel;
	}
	Ok(())
}
