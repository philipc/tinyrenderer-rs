use std::path::Path;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std;

#[derive(Default)]
struct Le16 {
	buf: [u8; 2]
}

impl From<Le16> for u16 {
	fn from(le16: Le16) -> Self {
		u16::from_le(unsafe { std::mem::transmute(le16.buf) })
	}
}

impl From<u16> for Le16 {
	fn from(n: u16) -> Self {
		Le16 {
			buf: unsafe { std::mem::transmute(n.to_le()) }
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
			std::slice::from_raw_parts(self as *const Self as *const u8, 
						   std::mem::size_of::<Self>())
		}
	}
}

const TGA_FOOTER: &'static [u8] = b"TRUEVISION-XFILE.\0";

#[derive(Default)]
#[repr(C, packed)]
pub struct TgaColor {
	b: u8,
	g: u8,
	r: u8,
	a: u8,
}

impl TgaColor {
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> TgaColor {
		TgaColor {
			b: b,
			g: g,
			r: r,
			a: a,
		}
	}

	fn from_u8(buf: &[u8]) -> Self {
		let a = if buf.len() >= 4 { buf[3] } else { 0 };
		TgaColor {
			b: buf[0],
			g: buf[1],
			r: buf[2],
			a: a,
		}
	}

	fn to_u8(&self, buf: &mut [u8]) {
		buf[0] = self.b;
		buf[1] = self.g;
		buf[2] = self.r;
		if buf.len() >= 4 {
			buf[3] = self.a;
		}
	}
}

/*

struct TGAColor {
	union {
		struct {
			unsigned char b, g, r, a;
		};
		unsigned char raw[4];
		unsigned int val;
	};
	int bytespp;

	TGAColor() : val(0), bytespp(1) {
	}

	TGAColor(unsigned char R, unsigned char G, unsigned char B, unsigned char A) : b(B), g(G), r(R), a(A), bytespp(4) {
	}

	TGAColor(int v, int bpp) : val(v), bytespp(bpp) {
	}

	TGAColor(const TGAColor &c) : val(c.val), bytespp(c.bytespp) {
	}

	TGAColor(const unsigned char *p, int bpp) : val(0), bytespp(bpp) {
		for (int i=0; i<bpp; i++) {
			raw[i] = p[i];
		}
	}

	TGAColor & operator =(const TGAColor &c) {
		if (this != &c) {
			bytespp = c.bytespp;
			val = c.val;
		}
		return *this;
	}
};
*/


#[derive(Copy, Clone, PartialEq)]
pub enum TgaFormat {
	GRAYSCALE = 1,
	RGB = 3,
	RGBA = 4,
}

pub struct TgaImage {
	data: Vec<u8>,
	width: usize,
	height: usize,
	format: TgaFormat,
}

/*
class TGAImage {
protected:
	bool   load_rle_data(std::ifstream &in);
	bool unload_rle_data(std::ofstream &out);
public:

	TGAImage();
	TGAImage(int w, int h, int bpp);
	TGAImage(const TGAImage &img);
	bool read_tga_file(const char *filename);
	bool write_tga_file(const char *filename, bool rle=true);
	bool flip_horizontally();
	bool flip_vertically();
	bool scale(int w, int h);
	TGAColor get(int x, int y);
	bool set(int x, int y, TGAColor c);
	~TGAImage();
	TGAImage & operator =(const TGAImage &img);
	int get_width();
	int get_height();
	int get_bytespp();
	unsigned char *buffer();
	void clear();
};

#endif //__IMAGE_H__
#include <iostream>
#include <fstream>
#include <string.h>
#include <time.h>
#include <math.h>
#include "tgaimage.h"

TGAImage::TGAImage() : data(NULL), width(0), height(0), bytespp(0) {
}
*/

impl TgaImage {
	pub fn new(width: usize, height: usize, format: TgaFormat) -> Self {
		let nbytes = width * height * format as usize;
		let data = vec![0; nbytes];
		TgaImage {
			data: data,
			width: width,
			height: height,
			format: format,
		}
	}
}

/*
TGAImage::TGAImage(const TGAImage &img) {
	width = img.width;
	height = img.height;
	bytespp = img.bytespp;
	unsigned long nbytes = width*height*bytespp;
	data = new unsigned char[nbytes];
	memcpy(data, img.data, nbytes);
}

TGAImage::~TGAImage() {
	if (data) delete [] data;
}

TGAImage & TGAImage::operator =(const TGAImage &img) {
	if (this != &img) {
		if (data) delete [] data;
		width  = img.width;
		height = img.height;
		bytespp = img.bytespp;
		unsigned long nbytes = width*height*bytespp;
		data = new unsigned char[nbytes];
		memcpy(data, img.data, nbytes);
	}
	return *this;
}

bool TGAImage::read_tga_file(const char *filename) {
	if (data) delete [] data;
	data = NULL;
	std::ifstream in;
	in.open (filename, std::ios::binary);
	if (!in.is_open()) {
		std::cerr << "can't open file " << filename << "\n";
		in.close();
		return false;
	}
	TGAHeader header;
	in.read((char *)&header, sizeof(header));
	if (!in.good()) {
		in.close();
		std::cerr << "an error occured while reading the header\n";
		return false;
	}
	width   = header.width;
	height  = header.height;
	bytespp = header.bitsperpixel>>3;
	if (width<=0 || height<=0 || (bytespp!=GRAYSCALE && bytespp!=RGB && bytespp!=RGBA)) {
		in.close();
		std::cerr << "bad bpp (or width/height) value\n";
		return false;
	}
	unsigned long nbytes = bytespp*width*height;
	data = new unsigned char[nbytes];
	if (3==header.datatypecode || 2==header.datatypecode) {
		in.read((char *)data, nbytes);
		if (!in.good()) {
			in.close();
			std::cerr << "an error occured while reading the data\n";
			return false;
		}
	} else if (10==header.datatypecode||11==header.datatypecode) {
		if (!load_rle_data(in)) {
			in.close();
			std::cerr << "an error occured while reading the data\n";
			return false;
		}
	} else {
		in.close();
		std::cerr << "unknown file format " << (int)header.datatypecode << "\n";
		return false;
	}
	if (!(header.imagedescriptor & 0x20)) {
		flip_vertically();
	}
	if (header.imagedescriptor & 0x10) {
		flip_horizontally();
	}
	std::cerr << width << "x" << height << "/" << bytespp*8 << "\n";
	in.close();
	return true;
}

bool TGAImage::load_rle_data(std::ifstream &in) {
	unsigned long pixelcount = width*height;
	unsigned long currentpixel = 0;
	unsigned long currentbyte  = 0;
	TGAColor colorbuffer;
	do {
		unsigned char chunkheader = 0;
		chunkheader = in.get();
		if (!in.good()) {
			std::cerr << "an error occured while reading the data\n";
			return false;
		}
		if (chunkheader<128) {
			chunkheader++;
			for (int i=0; i<chunkheader; i++) {
				in.read((char *)colorbuffer.raw, bytespp);
				if (!in.good()) {
					std::cerr << "an error occured while reading the header\n";
					return false;
				}
				for (int t=0; t<bytespp; t++)
					data[currentbyte++] = colorbuffer.raw[t];
				currentpixel++;
				if (currentpixel>pixelcount) {
					std::cerr << "Too many pixels read\n";
					return false;
				}
			}
		} else {
			chunkheader -= 127;
			in.read((char *)colorbuffer.raw, bytespp);
			if (!in.good()) {
				std::cerr << "an error occured while reading the header\n";
				return false;
			}
			for (int i=0; i<chunkheader; i++) {
				for (int t=0; t<bytespp; t++)
					data[currentbyte++] = colorbuffer.raw[t];
				currentpixel++;
				if (currentpixel>pixelcount) {
					std::cerr << "Too many pixels read\n";
					return false;
				}
			}
		}
	} while (currentpixel < pixelcount);
	return true;
}
*/

impl TgaImage {
	fn bytes_per_pixel(&self) -> usize {
		self.format as usize
	}

	fn bits_per_pixel(&self) -> usize {
		self.bytes_per_pixel() * 8
	}

	fn alpha_bits_per_pixel(&self) -> usize {
		if self.format == TgaFormat::RGBA {
			8
		} else {
			0
		}
	}

	pub fn write(&self, path: &Path, rle: bool) -> io::Result<()> {
		let mut file = BufWriter::new(try!(File::create(path)));

		let header = TgaHeader {
			image_type: if self.format == TgaFormat::GRAYSCALE { if rle { 11 } else { 3 } } else if rle { 10 } else { 2 },
			width: From::from(self.width as u16),
			height: From::from(self.height as u16),
			bits_per_pixel: self.bits_per_pixel() as u8,
			image_descriptor: 0x20 + self.alpha_bits_per_pixel() as u8 , // top-left origin
			.. Default::default()
		};
		try!(file.write_all(header.as_u8_slice()));

		if rle {
			try!(self.write_rle(&mut file));
		} else {
			try!(file.write_all(&self.data[..]));
		}

		try!(file.write_all(&[0; 4])); // developer area offset
		try!(file.write_all(&[0; 4])); // extension area offset
		try!(file.write_all(TGA_FOOTER));
		Ok(())
	}

	fn write_rle(&self, file: &mut Write) -> io::Result<()> {
		let bytes_per_pixel = self.bytes_per_pixel();
		let num_pixels = self.width * self.height;
		let mut start_pixel = 0;
		while start_pixel < num_pixels {
			let mut cur_color = &self.data[start_pixel * bytes_per_pixel..][..bytes_per_pixel];
			let mut raw = true;
			let mut run_length = 1;
			let mut next_pixel = start_pixel + 1;
			while next_pixel < num_pixels && run_length < 128 {
				let next_color = &self.data[next_pixel * bytes_per_pixel..][..bytes_per_pixel];
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
				try!(file.write_all(&self.data[start_pixel * bytes_per_pixel..][..run_length * bytes_per_pixel]));
			} else {
				try!(file.write_all(&[(run_length+127) as u8]));
				try!(file.write_all(cur_color));
			}
			start_pixel = next_pixel;
		}
		Ok(())
	}

	pub fn get(&self, x: usize, y: usize) -> TgaColor {
		if x < self.width && y < self.height {
			let offset = (x + y * self.width) * self.bytes_per_pixel();
			TgaColor::from_u8(&self.data[offset..][..self.bytes_per_pixel()])
		} else {
			TgaColor::new(0, 0, 0, 255)
		}
	}

	pub fn set(&mut self, x: usize, y: usize, color: &TgaColor) {
		let bytes_per_pixel = self.bytes_per_pixel();
		if x < self.width && y < self.height {
			let offset = (x + y * self.width) * bytes_per_pixel;
			color.to_u8(&mut self.data[offset..][..bytes_per_pixel]);
		}
	}

	pub fn flip_horizontally(&mut self) {
		let width = self.width;
		for i in 0 .. width/2 {
			for j in 0 .. self.height {
				let c1 = self.get(i, j);
				let c2 = self.get(width - 1 - i, j);
				self.set(i, j, &c2);
				self.set(width - 1 - i, j, &c1);
			}
		}
	}

	pub fn flip_vertically(&mut self) {
		let bytes_per_line = self.width * self.bytes_per_pixel();
		let half = self.height / 2;
		let (top, bot) = self.data.split_at_mut(half * bytes_per_line);
		let mut line = Vec::with_capacity(bytes_per_line);
		for j in 0 .. half {
			let line1 = &mut top[j * bytes_per_line ..][.. bytes_per_line];
			let line2 = &mut bot[(self.height - half - 1 - j) * bytes_per_line ..][.. bytes_per_line];
			line.extend_from_slice(line1);
			line1.clone_from_slice(line2);
			line2.clone_from_slice(&line[..]);
			line.truncate(0);
		}
	}
}

/*

unsigned char *TGAImage::buffer() {
	return data;
}

void TGAImage::clear() {
	memset((void *)data, 0, width*height*bytespp);
}

bool TGAImage::scale(int w, int h) {
	if (w<=0 || h<=0 || !data) return false;
	unsigned char *tdata = new unsigned char[w*h*bytespp];
	int nscanline = 0;
	int oscanline = 0;
	int erry = 0;
	unsigned long nlinebytes = w*bytespp;
	unsigned long olinebytes = width*bytespp;
	for (int j=0; j<height; j++) {
		int errx = width-w;
		int nx   = -bytespp;
		int ox   = -bytespp;
		for (int i=0; i<width; i++) {
			ox += bytespp;
			errx += w;
			while (errx>=(int)width) {
				errx -= width;
				nx += bytespp;
				memcpy(tdata+nscanline+nx, data+oscanline+ox, bytespp);
			}
		}
		erry += h;
		oscanline += olinebytes;
		while (erry>=(int)height) {
			if (erry>=(int)height<<1) // it means we jump over a scanline
				memcpy(tdata+nscanline+nlinebytes, tdata+nscanline, nlinebytes);
			erry -= height;
			nscanline += nlinebytes;
		}
	}
	delete [] data;
	data = tdata;
	width = w;
	height = h;
	return true;
}
const TGAColor white = TGAColor(255, 255, 255, 255);
const TGAColor red   = TGAColor(255, 0,   0,   255);
*/



impl TgaImage {
	pub fn line(&mut self, mut x0: usize, mut y0: usize, mut x1: usize, mut y1: usize, color: &TgaColor) {
		let mut dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
		let mut dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };
		let steep = dy > dx;
		if steep {
			std::mem::swap(&mut x0, &mut y0);
			std::mem::swap(&mut x1, &mut y1);
			std::mem::swap(&mut dx, &mut dy);
		};
		if x0 > x1 {
			std::mem::swap(&mut x0, &mut x1);
			std::mem::swap(&mut y0, &mut y1);
		}
		let mut x = x0;
		let mut y = y0;
		let mut d = dx;
		loop {
			if steep {
				self.set(y, x, color);
			} else {
				self.set(x, y, color);
			}
			if x == x1 {
				break;
			}
			x += 1;
			d += 2 * dy;
			if d > 2 * dx {
				if y0 < y1 {
					y += 1;
				} else {
					y -= 1;
				}
				d -= 2 * dx;
			}
		}
	}
}
