use tga;
use vec;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::num;
use std::path::Path;
use std::f64;

#[derive(Debug)]
pub enum ModelError {
	Io(io::Error),
	Parse(String),
}

impl From<io::Error> for ModelError {
	fn from(err: io::Error) -> ModelError {
		ModelError::Io(err)
	}
}

impl From<num::ParseFloatError> for ModelError {
	fn from(err: num::ParseFloatError) -> ModelError {
		ModelError::Parse(err.to_string())
	}
}

impl From<num::ParseIntError> for ModelError {
	fn from(err: num::ParseIntError) -> ModelError {
		ModelError::Parse(err.to_string())
	}
}

struct Face {
	vert: Vec<usize>,
	texture: Vec<usize>,
}

pub struct Model {
	vert: Vec<vec::Vec3<f64>>,
	texture: Vec<vec::Vec3<f64>>,
	face: Vec<Face>,
}

impl Model {
	pub fn new() -> Self {
		Model {
			vert: Vec::new(),
			texture: Vec::new(),
			face: Vec::new(),
		}
	}

	pub fn read(&mut self, path: &Path) -> Result<(), ModelError> {
		let file = BufReader::new(try!(File::open(path)));
		for line in file.lines() {
			let line = try!(line);
			let mut words = line.split_whitespace();
			match words.next() {
				Some("v") => self.vert.push(try!(Model::read_vert(&mut words))),
				Some("vt") => self.texture.push(try!(Model::read_vert(&mut words))),
				Some("f") => try!(self.read_face(&mut words)),
				Some(_) => (),
				None => (),
			}
		};
		Ok(())
	}

	fn read_vert<'a, I: Iterator<Item=&'a str>>(words: &mut I) -> Result<vec::Vec3<f64>, ModelError> {
		let x = try!(Model::read_f64(words));
		let y = try!(Model::read_f64(words));
		let z = try!(Model::read_f64(words));
		Ok(vec::Vec3::new(x, y, z))
	}

	fn read_f64<'a, I: Iterator<Item=&'a str>>(words: &mut I) -> Result<f64, ModelError> {
		match words.next() {
			Some(word) => Ok(try!(word.parse::<f64>())),
			None => Err(ModelError::Parse("missing f64".into())),
		}
	}

	fn read_face<'a, I: Iterator<Item=&'a str>>(&mut self, words: &mut I) -> Result<(), ModelError> {
		let mut vert = Vec::new();
		let mut texture = Vec::new();
		for word in words {
			let mut indices = word.split('/');
			vert.push(try!(Model::read_idx(indices.next(), self.vert.len())));
			texture.push(try!(Model::read_idx(indices.next(), self.texture.len())));
		}
		if vert.len() != 3 {
			return Err(ModelError::Parse("face must have exactly 3 vertices".into()));
		}
		self.face.push(Face {
			vert: vert,
			texture: texture,
		});
		Ok(())
	}

	fn read_idx(word_opt: Option<&str>, len: usize) -> Result<usize, ModelError> {
		let idx = match word_opt {
			Some(word) => try!(word.parse::<usize>()) - 1,
			None => return Err(ModelError::Parse("missing idx".into())),
		};
		if idx >= len {
			return Err(ModelError::Parse("face idx is too large".into()));
		};
		Ok(idx)
	}

	pub fn line(&self, image: &mut tga::TgaImage, x: i32, y: i32, w: i32, h: i32, color: &tga::TgaColor) {
		let width = w as f64;
		let height = h as f64;

		for face in &self.face {
			for (i, idx0) in face.vert.iter().enumerate() {
				let idx1 = face.vert.get(i + 1).unwrap_or(face.vert.first().unwrap());
				let v0 = &self.vert[*idx0];
				let v1 = &self.vert[*idx1];
				let p0 = vec::Vec2::new(
					x + ((v0.x + 1f64) * width / 2f64) as i32,
					y + ((v0.y + 1f64) * height / 2f64) as i32);
				let p1 = vec::Vec2::new(
					x + ((v1.x + 1f64) * width / 2f64) as i32,
					y + ((v1.y + 1f64) * height / 2f64) as i32);
				image.line(p0, p1, color);
			}
		}
	}

	#[allow(dead_code)]
	pub fn fill(&self, image: &mut tga::TgaImage, x: i32, y: i32, w: i32, h: i32) {
		let light_dir = vec::Vec3::new(0f64, 0f64, -1f64);
		let width = w as f64;
		let height = h as f64;

		for face in &self.face {
			let v0 = &self.vert[face.vert[0]];
			let v1 = &self.vert[face.vert[1]];
			let v2 = &self.vert[face.vert[2]];

			let intensity = (&v2.sub(v0)).cross(&v1.sub(v0)).normalize().dot(&light_dir);
			if intensity <= 0f64 {
				continue;
			}
			let intensity = (intensity * 255f64) as u8;
			let color = tga::TgaColor::new(intensity, intensity, intensity, 255);

			let p0 = vec::Vec2::new(
				x + ((v0.x + 1f64) * width / 2f64 + 0.5f64) as i32,
				y + ((v0.y + 1f64) * height / 2f64 + 0.5f64) as i32);
			let p1 = vec::Vec2::new(
				x + ((v1.x + 1f64) * width / 2f64 + 0.5f64) as i32,
				y + ((v1.y + 1f64) * height / 2f64 + 0.5f64) as i32);
			let p2 = vec::Vec2::new(
				x + ((v2.x + 1f64) * width / 2f64 + 0.5f64) as i32,
				y + ((v2.y + 1f64) * height / 2f64 + 0.5f64) as i32);

			image.fill(p0, p1, p2, &color);
		}
	}

	#[allow(dead_code)]
	pub fn fill_float(&self, image: &mut tga::TgaImage, texture: &tga::TgaImage,
			  x: i32, y: i32, w: i32, h: i32) {
		let light_dir = vec::Vec3::new(0f64, 0f64, -1f64);
		let width = w as f64;
		let height = h as f64;
		let mut zbuffer = vec![f64::MIN; image.get_width() * image.get_height()];

		for face in &self.face {
			let v0 = &self.vert[face.vert[0]];
			let v1 = &self.vert[face.vert[1]];
			let v2 = &self.vert[face.vert[2]];

			let t0 = &self.texture[face.texture[0]];
			let t1 = &self.texture[face.texture[1]];
			let t2 = &self.texture[face.texture[2]];

			let intensity = (&v2.sub(v0)).cross(&v1.sub(v0)).normalize().dot(&light_dir);
			if intensity <= 0f64 {
				continue;
			}

			let p0 = vec::Vec3::new(
				x as f64 + (v0.x + 1f64) * width / 2f64,
				y as f64 + (v0.y + 1f64) * height / 2f64,
				v0.z);
			let p1 = vec::Vec3::new(
				x as f64 + (v1.x + 1f64) * width / 2f64,
				y as f64 + (v1.y + 1f64) * height / 2f64,
				v1.z);
			let p2 = vec::Vec3::new(
				x as f64 + (v2.x + 1f64) * width / 2f64,
				y as f64 + (v2.y + 1f64) * height / 2f64,
				v2.z);

			image.fill_float(&p0, &p1, &p2, t0, t1, t2, intensity, texture, &mut zbuffer[..]);
		}
	}
}

