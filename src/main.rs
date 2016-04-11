extern crate image;

mod model;
mod tga;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
	let mut model = model::Model::new();
	model.read(Path::new("african_head.obj")).unwrap();

	let white = &tga::TgaColor::new(255, 255, 255, 255);
	let red = &tga::TgaColor::new(255, 0, 0, 255);

	let width = 800;
	let height = 800;
	let mut image = tga::TgaImage::new(width, height, tga::TgaFormat::RGB);
	model.line(&mut image, 0, 0, width, height, white);
	image.flip_vertically();

	let path = Path::new("output.tga");
	image.write(path, true).unwrap();

	/*
	let reader = File::open(path).unwrap();
	let reader = BufReader::new(reader);
	image::load(reader, image::ImageFormat::TGA).unwrap();
	*/
}
