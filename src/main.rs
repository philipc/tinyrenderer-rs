extern crate image;

mod model;
mod tga;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
	//let white = &tga::TgaColor::new(255, 255, 255, 255);
	//let red = &tga::TgaColor::new(255, 0, 0, 255);
	//let green = &tga::TgaColor::new(0, 255, 0, 255);

	let width = 800;
	let height = 800;
	let mut image = tga::TgaImage::new(width, height, tga::TgaFormat::RGB);

	let mut model = model::Model::new();
	model.read(Path::new("african_head.obj")).unwrap();
	model.fill(&mut image, 0, 0, width, height);

	//image.fill(10, 70, 50, 160, 70, 80, red);
	//image.fill(180, 50, 150, 1, 70, 180, white);
	//image.fill(180, 150, 120, 160, 130, 180, green);
	//image.fill2(10, 70, 50, 160, 70, 80, red);
	//image.fill2(180, 50, 150, 1, 70, 180, white);
	//image.fill2(180, 150, 120, 160, 130, 180, green);
	image.flip_vertically();

	let path = Path::new("output.tga");
	image.write(path, true).unwrap();

	/*
	let reader = File::open(path).unwrap();
	let reader = BufReader::new(reader);
	image::load(reader, image::ImageFormat::TGA).unwrap();
	*/
}
