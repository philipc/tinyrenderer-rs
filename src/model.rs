use tga;

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

pub struct Vec3<T> {
        pub x: T,
        pub y: T,
        pub z: T,
}

impl<T> Vec3<T> {
        pub fn new(x: T, y: T, z: T) -> Self {
                Vec3 {
                        x: x,
                        y: y,
                        z: z,
                }
        }
}

pub struct Model {
        verts: Vec<Vec3<f64>>,
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
                let vert = Vec3::new(x, y, z);
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

        pub fn vert(&self, idx: usize) -> &Vec3<f64> {
                &self.verts[idx]
        }

        pub fn nfaces(&self) -> usize {
                self.faces.len()
        }

        pub fn faces(&self) -> &Vec<Vec<usize>> {
                &self.faces
        }

        pub fn line(&self, image: &mut tga::TgaImage, x: usize, y: usize, w: usize, h: usize, color: &tga::TgaColor) {
                let width = w as f64;
                let height = h as f64;

                for face in &self.faces {
                        for (i, idx0) in face.iter().enumerate() {
                                let idx1 = face.get(i + 1).unwrap_or(face.first().unwrap());
                                let v0 = &self.verts[*idx0];
                                let v1 = &self.verts[*idx1];
                                let x0 = x + ((v0.x + 1f64) * width / 2f64) as usize;
                                let y0 = y + ((v0.y + 1f64) * width / 2f64) as usize;
                                let x1 = x + ((v1.x + 1f64) * width / 2f64) as usize;
                                let y1 = y + ((v1.y + 1f64) * width / 2f64) as usize;
                                image.line(x0, y0, x1, y1, color);
                        }
                }
        }
}

