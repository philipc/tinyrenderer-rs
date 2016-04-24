use tga;
use vec;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::num;
use std::path::Path;
use std::str;

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

pub struct Model {
	verts: Vec<vec::Vec3<f64>>,
	faces: Vec<Vec<usize>>,
}

impl Model {
	pub fn new() -> Self {
		Model {
			verts: Vec::new(),
			faces: Vec::new(),
		}
	}

	pub fn read(&mut self, path: &Path) -> Result<(), ModelError> {
		let mut file = BufReader::new(try!(File::open(path)));
		for line in file.lines() {
			let line = try!(line);
			let mut words = line.split_whitespace();
			match words.next() {
				Some("v") => try!(self.read_vert(&mut words)),
				Some("f") => try!(self.read_face(&mut words)),
				Some(_) => (),
				None => (),
			}
		};
		Ok(())
	}

	fn read_vert<'a, I: Iterator<Item=&'a str>>(&mut self, words: &mut I) -> Result<(), ModelError> {
		let x = try!(self.read_f64(words));
		let y = try!(self.read_f64(words));
		let z = try!(self.read_f64(words));
		let vert = vec::Vec3::new(x, y, z);
		self.verts.push(vert);
		Ok(())
	}

	fn read_f64<'a, I: Iterator<Item=&'a str>>(&mut self, words: &mut I) -> Result<f64, ModelError> {
		match words.next() {
			Some(word) => Ok(try!(word.parse::<f64>())),
			None => Err(ModelError::Parse("missing f64".into())),
		}
	}

	fn read_face<'a, I: Iterator<Item=&'a str>>(&mut self, words: &mut I) -> Result<(), ModelError> {
		let mut verts = Vec::new();
		for word in words {
			verts.push(try!(self.read_idx(word)));
		}
		if verts.len() < 3 {
			return Err(ModelError::Parse("too few vertices in face".into()));
		}
		self.faces.push(verts);
		Ok(())
	}

	fn read_idx(&self, word: &str) -> Result<usize, ModelError> {
		let idx = match word.split('/').next() {
			Some(word) => try!(word.parse::<usize>()) - 1,
			None => return Err(ModelError::Parse("missing idx".into())),
		};
		if idx >= self.verts.len() {
			return Err(ModelError::Parse("face idx is too large".into()));
		};
		Ok(idx)
	}

	pub fn nverts(&self) -> usize {
		self.verts.len()
	}

	pub fn vert(&self, idx: usize) -> &vec::Vec3<f64> {
		&self.verts[idx]
	}

	pub fn nfaces(&self) -> usize {
		self.faces.len()
	}

	pub fn faces(&self) -> &Vec<Vec<usize>> {
		&self.faces
	}

	pub fn line(&self, image: &mut tga::TgaImage, x: i32, y: i32, w: i32, h: i32, color: &tga::TgaColor) {
		let width = w as f64;
		let height = h as f64;

		for face in &self.faces {
			for (i, idx0) in face.iter().enumerate() {
				let idx1 = face.get(i + 1).unwrap_or(face.first().unwrap());
				let v0 = &self.verts[*idx0];
				let v1 = &self.verts[*idx1];
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

	pub fn fill(&self, image: &mut tga::TgaImage, x: i32, y: i32, w: i32, h: i32) {
		let light_dir = vec::Vec3::new(0f64, 0f64, -1f64);
		let width = w as f64;
		let height = h as f64;

		for face in &self.faces {
			let v0 = &self.verts[face[0]];
			let v1 = &self.verts[face[1]];
			let v2 = &self.verts[face[2]];

			let intensity = (&v2.sub(v0)).cross(&v1.sub(v0)).normalize().dot(&light_dir);
			if intensity <= 0f64 {
				continue;
			}
			let intensity = (intensity * 255f64) as u8;
			let color = tga::TgaColor::new(intensity, intensity, intensity, 255);

			let p0 = vec::Vec2::new(
				x + ((v0.x + 1f64) * width / 2f64) as i32,
				y + ((v0.y + 1f64) * height / 2f64) as i32);
			let p1 = vec::Vec2::new(
				((v1.x + 1f64) * width / 2f64) as i32,
				((v1.y + 1f64) * height / 2f64) as i32);
			let p2 = vec::Vec2::new(
				((v2.x + 1f64) * width / 2f64) as i32,
				((v2.y + 1f64) * height / 2f64) as i32);

			image.fill(p0, p1, p2, &color);
		}
	}
}

