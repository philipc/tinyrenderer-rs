use vec;

use std::{ cmp, mem };

#[derive(Default)]
#[repr(C, packed)]
pub struct Color {
	b: u8,
	g: u8,
	r: u8,
	a: u8,
}

impl Color {
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
		Color {
			b: b,
			g: g,
			r: r,
			a: a,
		}
	}

	fn from_u8(buf: &[u8]) -> Self {
		let a = if buf.len() >= 4 { buf[3] } else { 0 };
		Color {
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

	pub fn intensity(&self, intensity: f64) -> Self {
		Color {
			r: (self.r as f64 * intensity).round() as u8,
			g: (self.g as f64 * intensity).round() as u8,
			b: (self.b as f64 * intensity).round() as u8,
			a: 255,
		}
	}
}

#[derive(Copy, Clone, PartialEq)]
pub enum Format {
	Gray,
	Rgb,
	Rgba,
}

impl Format {
        pub fn bytes_per_pixel(&self) -> usize {
                match *self {
                        Format::Gray => 1,
                        Format::Rgb => 3,
                        Format::Rgba => 4,
                }
        }

        pub fn alpha_bytes_per_pixel(&self) -> usize {
                match *self {
                        Format::Gray => 0,
                        Format::Rgb => 0,
                        Format::Rgba => 1,
                }
        }
}

pub struct Image {
	data: Vec<u8>,
	width: usize,
	height: usize,
	format: Format,
}

impl Image {
	pub fn new(width: usize, height: usize, format: Format) -> Self {
		let nbytes = width * height * format.bytes_per_pixel();
		let data = vec![0; nbytes];
		Image {
			data: data,
			width: width,
			height: height,
			format: format,
		}
	}

	pub fn from_data(width: usize, height: usize, format: Format, data: Vec<u8>) -> Self {
		Image {
			data: data,
			width: width,
			height: height,
			format: format,
		}
        }

	pub fn get_data(&self) -> &Vec<u8> {
                &self.data
        }

	pub fn get_width(&self) -> usize {
		self.width
	}

	pub fn get_height(&self) -> usize {
		self.height
	}

        pub fn get_format(&self) -> Format {
                self.format
        }

	pub fn get(&self, x: usize, y: usize) -> Color {
		if x < self.width && y < self.height {
		        let bytes_per_pixel = self.format.bytes_per_pixel();
			let offset = (x + y * self.width) * bytes_per_pixel;
			Color::from_u8(&self.data[offset..][..bytes_per_pixel])
		} else {
			Color::new(0, 0, 0, 255)
		}
	}

	pub fn set(&mut self, x: usize, y: usize, color: &Color) {
		if x < self.width && y < self.height {
		        let bytes_per_pixel = self.format.bytes_per_pixel();
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
		let bytes_per_line = self.width * self.format.bytes_per_pixel();
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

impl Image {
	#[allow(dead_code)]
	pub fn line<'a>(&mut self,
		    p0: &'a vec::Vec2<i32>,
		    p1: &'a vec::Vec2<i32>,
		    color: &Color) {
		let (mut x0, mut y0) = p0.as_tuple();
		let (mut x1, mut y1) = p1.as_tuple();
		let dx = (x1 - x0).abs();
		let dy = (y1 - y0).abs();
		let steep = dy > dx;
		if steep {
			mem::swap(&mut x0, &mut y0);
			mem::swap(&mut x1, &mut y1);
		};
		if x0 > x1 {
			mem::swap(&mut x0, &mut x1);
			mem::swap(&mut y0, &mut y1);
		}
		let mut s = LineState::new(x0, y0, x1, y1, LineStateRound::Nearest);
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
		    color: &Color) {
		if x0 > x1 {
			mem::swap(&mut x0, &mut x1);
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
	pub fn triangle(&mut self,
		    mut p0: vec::Vec2<i32>,
		    mut p1: vec::Vec2<i32>,
		    mut p2: vec::Vec2<i32>,
		    color: &Color) {
		if p0.0[1] == p1.0[1] && p0.0[1] == p2.0[1] {
			return;
		}
		if p0.0[1] > p1.0[1] {
			mem::swap(&mut p0, &mut p1);
		}
		if p0.0[1] > p2.0[1] {
			mem::swap(&mut p0, &mut p2);
		}
		if p1.0[1] > p2.0[1] {
			mem::swap(&mut p1, &mut p2);
		}
		let (mut roundl, mut roundr) = (LineStateRound::Left, LineStateRound::Right);
		let order = (p2.0[0] - p0.0[0]) * (p1.0[1] - p0.0[1]) - (p1.0[0] - p0.0[0]) * (p2.0[1] - p0.0[1]);
		if order < 0 {
			mem::swap(&mut roundl, &mut roundr);
		}
		let mut l = LineState::new(p0.0[1], p0.0[0], p1.0[1], p1.0[0], roundl);
		let mut r = LineState::new(p0.0[1], p0.0[0], p2.0[1], p2.0[0], roundr);
		loop {
			if (r.b - l.b) * order >= 0 {
				self.horizontal_line(l.b, r.b, r.a, color);
			}
			if !l.step() {
				break;
			}
			r.step();
		}
		l = LineState::new(p1.0[1], p1.0[0], p2.0[1], p2.0[0], roundl);
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
	#[allow(dead_code)]
	fn inside(&self, p: &vec::Vec2<i32>,
		  p0: &vec::Vec2<i32>,
		  p1: &vec::Vec2<i32>,
		  p2: &vec::Vec2<i32>)
		  -> bool {
		let v1 = vec::Vec3::new(p1.0[0] - p0.0[0], p2.0[0] - p0.0[0], p0.0[0] - p.0[0]);
		let v2 = vec::Vec3::new(p1.0[1] - p0.0[1], p2.0[1] - p0.0[1], p0.0[1] - p.0[1]);
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
		let v1 = vec::Vec3::new(p1.0[0] - p0.0[0], p2.0[0] - p0.0[0], p0.0[0] - p.0[0]);
		let v2 = vec::Vec3::new(p1.0[1] - p0.0[1], p2.0[1] - p0.0[1], p0.0[1] - p.0[1]);
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

	pub fn render(&mut self, shader: &Shader, texture: &Image,
		      p0: &vec::Vec3<f64>, p1: &vec::Vec3<f64>, p2: &vec::Vec3<f64>,
		      zbuffer: &mut [f64]) {
		let minx = cmp::max(0, p0.0[0].min(p1.0[0].min(p2.0[0])).ceil() as i32);
		let miny = cmp::max(0, p0.0[1].min(p1.0[1].min(p2.0[1])).ceil() as i32);
		let maxx = cmp::min(self.width as i32 - 1, p0.0[0].max(p1.0[0].max(p2.0[0])).floor() as i32);
		let maxy = cmp::min(self.height as i32 - 1, p0.0[1].max(p1.0[1].max(p2.0[1])).floor() as i32);
		for y in miny .. maxy + 1 {
			for x in minx .. maxx + 1 {
				match self.barycentric(&vec::Vec2::new(x as f64, y as f64), p0, p1, p2) {
					None => (),
					Some(bc) => {
						let z = p0.0[2] * bc.0[0] + p1.0[2] * bc.0[1] + p2.0[2] * bc.0[2];
						if zbuffer[x as usize + y as usize * self.width] < z {
							match shader.fragment(&bc, texture) {
								None => (),
								Some(color) => {
									zbuffer[x as usize + y as usize * self.width] = z;
									self.set(x as usize, y as usize, &color);
								}
							}
						}
					}
				}
			}
		}
	}
}

pub trait Shader {
	fn vertex(&mut self, i: usize, vert: &vec::Vec3<f64>, uv: &vec::Vec3<f64>, normal: &vec::Vec3<f64>) -> vec::Vec3<f64>;
	fn fragment(&self, bc: &vec::Vec3<f64>, texture: &Image) -> Option<Color>;
}
