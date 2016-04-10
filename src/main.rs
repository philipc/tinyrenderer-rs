extern crate image;

mod model;
mod tga;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
	let mut model = model::Model::new();
	model.read(Path::new("african_head.obj")).unwrap();
	println!("{}", model.nverts());
	println!("{}", model.nfaces());

	let white = &tga::TgaColor::new(255, 255, 255, 255);
	let red = &tga::TgaColor::new(255, 0, 0, 255);

	let width = 800;
	let height = 800;
	let mut image = tga::TgaImage::new(width, height, tga::TgaFormat::RGB);

	for face in model.faces() {
		for (i, idx0) in face.iter().enumerate() {
			let idx1 = face.get(i + 1).unwrap_or(face.first().unwrap());
			let v0 = model.vert(*idx0);
			let v1 = model.vert(*idx1);
			let x0 = ((v0.x + 1f64) * width as f64 / 2f64) as usize;
			let y0 = ((v0.y + 1f64) * width as f64 / 2f64) as usize;
			let x1 = ((v1.x + 1f64) * width as f64 / 2f64) as usize;
			let y1 = ((v1.y + 1f64) * width as f64 / 2f64) as usize;
			image.line(x0, y0, x1, y1, white);
		}
	}
	image.flip_vertically();

	let path = Path::new("output.tga");
	image.write(path, true).unwrap();

	/*
	let reader = File::open(path).unwrap();
	let reader = BufReader::new(reader);
	image::load(reader, image::ImageFormat::TGA).unwrap();
	*/
}
