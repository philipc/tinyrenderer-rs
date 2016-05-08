use image;
use image::Shader;
use vec;

use std::{ f64, fs, io, num, path };
use std::io::BufRead;

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
	normal: Vec<vec::Vec3<f64>>,
	texture: Vec<vec::Vec3<f64>>,
	face: Vec<Face>,
}

impl Default for Model {
	fn default() -> Self {
		Model {
			vert: Vec::new(),
			normal: Vec::new(),
			texture: Vec::new(),
			face: Vec::new(),
		}
	}
}

impl Model {
	pub fn read(path: &path::Path) -> Result<Model, ModelError> {
		let file = io::BufReader::new(try!(fs::File::open(path)));
		let mut model = Model::default();
		for line in file.lines() {
			let line = try!(line);
			let mut words = line.split_whitespace();
			match words.next() {
				Some("v") => model.vert.push(try!(Model::read_vert(&mut words))),
				Some("vn") => model.normal.push(try!(Model::read_vert(&mut words))),
				Some("vt") => model.texture.push(try!(Model::read_vert(&mut words))),
				Some("f") => model.face.push(try!(Model::read_face(&mut words, model.vert.len(), model.texture.len()))),
				_ => (),
			}
		};
		Ok(model)
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

	fn read_face<'a, I: Iterator<Item=&'a str>>(words: &mut I, vert_len: usize, texture_len: usize) -> Result<Face, ModelError> {
		let mut face = Face {
			vert: Vec::new(),
			texture: Vec::new(),
		};
		for word in words {
			let mut indices = word.split('/');
			face.vert.push(try!(Model::read_idx(indices.next(), vert_len)));
			face.texture.push(try!(Model::read_idx(indices.next(), texture_len)));
		}
		if face.vert.len() != 3 {
			return Err(ModelError::Parse("face must have exactly 3 vertices".into()));
		}
		Ok(face)
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

	#[allow(dead_code)]
	pub fn wireframe(&self, image: &mut image::Image, x: i32, y: i32, w: i32, h: i32, color: &image::Color) {
		let width = w as f64;
		let height = h as f64;

		for face in &self.face {
			for (i, idx0) in face.vert.iter().enumerate() {
				let idx1 = face.vert.get(i + 1).unwrap_or(face.vert.first().unwrap());
				let v0 = &self.vert[*idx0];
				let v1 = &self.vert[*idx1];
				let p0 = &vec::Vec2::new(
					x + ((v0.0[0] + 1f64) * width / 2f64) as i32,
					y + ((v0.0[1] + 1f64) * height / 2f64) as i32);
				let p1 = &vec::Vec2::new(
					x + ((v1.0[0] + 1f64) * width / 2f64) as i32,
					y + ((v1.0[1] + 1f64) * height / 2f64) as i32);
				image.line(p0, p1, color);
			}
		}
	}

	pub fn render(&self, image: &mut image::Image, shader: &mut image::Shader, texture: &image::Image) {
		let mut zbuffer = vec![f64::MIN; image.get_width() * image.get_height()];

		for face in &self.face {
			let p0 = &shader.vertex(0, &self.vert[face.vert[0]], &self.texture[face.texture[0]], &self.normal[face.vert[0]]);
			let p1 = &shader.vertex(1, &self.vert[face.vert[1]], &self.texture[face.texture[1]], &self.normal[face.vert[1]]);
			let p2 = &shader.vertex(2, &self.vert[face.vert[2]], &self.texture[face.texture[2]], &self.normal[face.vert[2]]);
			image.render(shader, texture, p0, p1, p2, &mut zbuffer[..]);
		}
	}
}

