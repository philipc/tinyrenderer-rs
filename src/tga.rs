use vec;

use std::cmp;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
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

	fn as_u8_slice_mut(&mut self) -> &mut [u8] {
		unsafe {
			std::slice::from_raw_parts_mut(self as *mut Self as *mut u8, 
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

	fn intensity(&self, intensity: f64) -> Self {
		TgaColor {
			r: (self.r as f64 * intensity).round() as u8,
			g: (self.g as f64 * intensity).round() as u8,
			b: (self.b as f64 * intensity).round() as u8,
			a: 255,
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

	#[allow(dead_code)]
	pub fn get_width(&self) -> usize {
		self.width
	}

	#[allow(dead_code)]
	pub fn get_height(&self) -> usize {
		self.height
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
*/

impl TgaImage {
	pub fn read(path: &Path) -> io::Result<Self> {
		let mut file = BufReader::new(try!(File::open(path)));
		let mut header = TgaHeader::default();
		try!(file.read_exact(header.as_u8_slice_mut()));
		let width: u16 = header.width.into();
		let height: u16 = header.height.into();
		let (format, rle) = match (header.image_type, header.bits_per_pixel) {
			(2, 24) => (TgaFormat::RGB, false),
			(2, 32) => (TgaFormat::RGBA, false),
			(3, 8) => (TgaFormat::GRAYSCALE, false),
			(10, 24) => (TgaFormat::RGB, true),
			(10, 32) => (TgaFormat::RGBA, true),
			(11, 8) => (TgaFormat::GRAYSCALE, true),
			_ => return Err(io::Error::new(io::ErrorKind::Other, "invalid image_type")),
		};
		// FIXME: avoid zero init of data
		let mut image = TgaImage::new(width as usize, height as usize, format);
		if rle {
			try!(image.read_rle(&mut file));
		} else {
			try!(file.read_exact(&mut image.data));
		}
		if header.image_descriptor & 0x10 != 0 {
			image.flip_horizontally();
		}
		if header.image_descriptor & 0x20 != 0{
			image.flip_vertically();
		}
		Ok(image)
	}
}

/*
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
*/

impl TgaImage {
	fn read_rle(&mut self, file: &mut Read) -> io::Result<()> {
		let bytes_per_pixel = self.bytes_per_pixel();
		let num_pixels = self.width * self.height;
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
				try!(file.read_exact(&mut self.data[start_pixel * bytes_per_pixel..][..run_length * bytes_per_pixel]));
				start_pixel = next_pixel;
			} else {
				// FIXME: read directly into self.data
				let mut color = vec![0; bytes_per_pixel];
				try!(file.read_exact(&mut color));
				while start_pixel < next_pixel {
					self.data[start_pixel * bytes_per_pixel..][..bytes_per_pixel].clone_from_slice(&color);
					start_pixel += 1;
				}
			}
		}
		Ok(())
	}
}

/*
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
			image_descriptor: self.alpha_bits_per_pixel() as u8,
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

	#[allow(dead_code)]
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

	#[allow(dead_code)]
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

#[derive(Clone, Copy, Debug)]
enum LineStateRound {
	Left,
	Nearest,
	Right,
}

struct LineState {
	pub a: i32,
	a_final: i32,
	pub b: i32,
	b_frac: i32, // fraction = b_frac/da
	da: i32,
	db: i32,
	dir: i32,
}

impl LineState {
	pub fn new(a0: i32, b0: i32, a1: i32, b1: i32, round: LineStateRound) -> Self {
		// double these so that we can implement rounding with integers
		let da = (a1 - a0).abs() * 2;
		let db = (b1 - b0).abs() * 2;
		let up = b1 > b0;
		let b_frac = match round {
			LineStateRound::Left => if up { da - 1 } else { 1 },
			LineStateRound::Nearest => da / 2,
			LineStateRound::Right => if up { 1 } else { da - 1 },
		};
		let dir = if up { 1 } else { -1 };
		LineState {
			a: a0,
			a_final: a1,
			b: b0,
			b_frac: b_frac,
			da: da,
			db: db,
			dir: dir,
		}
	}

	// step a by '1'
	// step b by 'db/da'
	pub fn step(&mut self) -> bool {
		if self.a == self.a_final {
			return false;
		}
		self.a += 1;
		self.b_frac += self.db;
		while self.b_frac >= self.da {
			self.b_frac -= self.da;
			self.b += self.dir;
		}
		true
	}
}

impl TgaImage {
	pub fn line(&mut self,
		    mut p0: vec::Vec2<i32>,
		    mut p1: vec::Vec2<i32>,
		    color: &TgaColor) {
		let dx = (p1.x - p0.x).abs();
		let dy = (p1.y - p0.y).abs();
		let steep = dy > dx;
		if steep {
			std::mem::swap(&mut p0.x, &mut p0.y);
			std::mem::swap(&mut p1.x, &mut p1.y);
		};
		if p0.x > p1.x {
			std::mem::swap(&mut p0.x, &mut p1.x);
			std::mem::swap(&mut p0.y, &mut p1.y);
		}
		let mut s = LineState::new(p0.x, p0.y, p1.x, p1.y, LineStateRound::Nearest);
		loop {
			if steep {
				self.set(s.b as usize, s.a as usize, color);
			} else {
				self.set(s.a as usize, s.b as usize, color);
			}
			if !s.step() {
				break;
			}
		}
	}

	pub fn horizontal_line(&mut self,
		    mut x0: i32, mut x1: i32, y: i32,
		    color: &TgaColor) {
		if x0 > x1 {
			std::mem::swap(&mut x0, &mut x1);
		}
		let mut x = x0;
		loop {
			self.set(x as usize, y as usize, color);
			if x == x1 {
				break;
			}
			x += 1;
		}
	}

	#[allow(dead_code)]
	pub fn fill(&mut self,
		    mut p0: vec::Vec2<i32>,
		    mut p1: vec::Vec2<i32>,
		    mut p2: vec::Vec2<i32>,
		    color: &TgaColor) {
		if p0.y == p1.y && p0.y == p2.y {
			return;
		}
		if p0.y > p1.y {
			std::mem::swap(&mut p0, &mut p1);
		}
		if p0.y > p2.y {
			std::mem::swap(&mut p0, &mut p2);
		}
		if p1.y > p2.y {
			std::mem::swap(&mut p1, &mut p2);
		}
		let (mut roundl, mut roundr) = (LineStateRound::Left, LineStateRound::Right);
		let order = (p2.x - p0.x) * (p1.y - p0.y) - (p1.x - p0.x) * (p2.y - p0.y);
		if order < 0 {
			std::mem::swap(&mut roundl, &mut roundr);
		}
		let mut l = LineState::new(p0.y, p0.x, p1.y, p1.x, roundl);
		let mut r = LineState::new(p0.y, p0.x, p2.y, p2.x, roundr);
		loop {
			if (r.b - l.b) * order >= 0 {
				self.horizontal_line(l.b, r.b, r.a, color);
			}
			if !l.step() {
				break;
			}
			r.step();
		}
		l = LineState::new(p1.y, p1.x, p2.y, p2.x, roundl);
		loop {
			if !l.step() {
				break;
			}
			r.step();
			if (r.b - l.b) * order >= 0 {
				self.horizontal_line(l.b, r.b, r.a, color);
			}
		}
	}

	// Return true if barycentric coordinates are >= 0
	// Define l0, l1, l2:
	// x = l0 x0 + l1 x1 + l2 x2
	// y = l0 y0 + l1 y1 + l2 y2
	// l0 + l1 + l2 = 1
	// Therefore:
	// 0 = l1 (x1 - x0) + l2 (x2 - x0) + (x0 - x)
	// 0 = l1 (y1 - y0) + l2 (y2 - y0) + (y0 - y)
	// Solve using cross product.
	fn inside(&self, p: vec::Vec2<i32>,
		  p0: vec::Vec2<i32>,
		  p1: vec::Vec2<i32>,
		  p2: vec::Vec2<i32>)
		  -> bool {
		let v1 = vec::Vec3::new(p1.x - p0.x, p2.x - p0.x, p0.x - p.x);
		let v2 = vec::Vec3::new(p1.y - p0.y, p2.y - p0.y, p0.y - p.y);
		let (l1, l2, scale) = v1.cross(&v2).as_tuple();
		if scale == 0 {
			return false;
		}
		let l0 = scale - l1 - l2;
		if scale > 0 {
			l0 >= 0 && l1 >= 0 && l2 >= 0
		} else {
			l0 <= 0 && l1 <= 0 && l2 <= 0
		}
	}

	fn barycentric(&self, p: &vec::Vec2<f64>,
		  p0: &vec::Vec3<f64>,
		  p1: &vec::Vec3<f64>,
		  p2: &vec::Vec3<f64>)
		  -> Option<vec::Vec3<f64>> {
		let v1 = vec::Vec3::new(p1.x - p0.x, p2.x - p0.x, p0.x - p.x);
		let v2 = vec::Vec3::new(p1.y - p0.y, p2.y - p0.y, p0.y - p.y);
		let (l1, l2, scale) = v1.cross(&v2).as_tuple();
		if scale == 0f64 {
			return None;
		}
		let l1 = l1 / scale;
		let l2 = l2 / scale;
		let l0 = 1f64 - l1 - l2;
		if l0 < 0f64 || l1 < 0f64 || l2 < 0f64 {
			return None;
		}
		Some(vec::Vec3::new(l0, l1, l2))
	}

	#[allow(dead_code)]
	pub fn fill2(&mut self,
		     p0: vec::Vec2<i32>,
		     p1: vec::Vec2<i32>,
		     p2: vec::Vec2<i32>,
		     color: &TgaColor) {
		let minx = cmp::max(0, cmp::min(p0.x, cmp::min(p1.x, p2.x)));
		let miny = cmp::max(0, cmp::min(p0.y, cmp::min(p1.y, p2.y)));
		let maxx = cmp::min(self.width as i32 - 1, cmp::max(p0.x, cmp::max(p1.x, p2.x)));
		let maxy = cmp::min(self.height as i32 - 1, cmp::max(p0.y, cmp::max(p1.y, p2.y)));
		for y in miny .. maxy + 1 {
			for x in minx .. maxx + 1 {
				if self.inside(vec::Vec2::new(x, y), p0, p1, p2) {
					self.set(x as usize, y as usize, color);
				}
			}
		}
	}

	pub fn fill_float(&mut self,
		     p0: &vec::Vec3<f64>, p1: &vec::Vec3<f64>, p2: &vec::Vec3<f64>,
		     t0: &vec::Vec3<f64>, t1: &vec::Vec3<f64>, t2: &vec::Vec3<f64>,
		     intensity: f64, texture: &TgaImage, zbuffer: &mut [f64]) {
		let minx = cmp::max(0, p0.x.min(p1.x.min(p2.x)).ceil() as i32);
		let miny = cmp::max(0, p0.y.min(p1.y.min(p2.y)).ceil() as i32);
		let maxx = cmp::min(self.width as i32 - 1, p0.x.max(p1.x.max(p2.x)).floor() as i32);
		let maxy = cmp::min(self.height as i32 - 1, p0.y.max(p1.y.max(p2.y)).floor() as i32);
		for y in miny .. maxy + 1 {
			for x in minx .. maxx + 1 {
				match self.barycentric(&vec::Vec2::new(x as f64, y as f64), p0, p1, p2) {
					None => (),
					Some(bc) => {
						let z = p0.z * bc.x + p1.z * bc.y + p2.z * bc.z;
						if zbuffer[x as usize + y as usize * self.width] < z {
							zbuffer[x as usize + y as usize * self.width] = z;
							let diffuse_x = ((t0.x * bc.x + t1.x * bc.y + t2.x * bc.z) * texture.get_width() as f64).floor() as usize;
							let diffuse_y = ((t0.y * bc.x + t1.y * bc.y + t2.y * bc.z) * texture.get_height() as f64).floor() as usize;
							let color = texture.get(diffuse_x, diffuse_y).intensity(intensity);
							self.set(x as usize, y as usize, &color);
						}
					}
				}
			}
		}
	}
}
