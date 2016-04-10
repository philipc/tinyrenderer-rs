mod tga;

use std::path::Path;

fn main() {
	let white = &tga::TgaColor::new(255, 255, 255, 255);
	let red = &tga::TgaColor::new(255, 0, 0, 255);

	let path = Path::new("output.tga");

	let mut image = tga::TgaImage::new(100, 100, tga::TgaFormat::RGB);
	image.line(13, 20, 80, 40, white);
	image.line(20, 13, 40, 80, red);
	image.line(85, 45, 18, 25, white);
	image.line(45, 85, 25, 18, red);
	image.line(30, 30, 70, 70, white);
	image.line(30, 70, 70, 30, red);
	image.flip_vertically();
	image.write(path, true).unwrap();

	/*
	let reader = File::open(path).unwrap();
	let reader = BufReader::new(reader);
	image::load(reader, image::ImageFormat::TGA).unwrap();
	*/
}
