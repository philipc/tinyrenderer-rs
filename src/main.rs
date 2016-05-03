extern crate image;

mod model;
mod tga;
mod vec;

use std::path::Path;

#[allow(dead_code)]
enum Lesson {
	L1Model,
	L2Triangle,
	L2Model,
	L3Model,
	L4Model,
}

fn main() {
	let white = &tga::TgaColor::new(255, 255, 255, 255);
	let red = &tga::TgaColor::new(255, 0, 0, 255);
	let green = &tga::TgaColor::new(0, 255, 0, 255);

	//let lesson = Lesson::L1Model;
	//let lesson = Lesson::L2Triangle;
	//let lesson = Lesson::L2Model;
	//let lesson = Lesson::L3Model;
	let lesson = Lesson::L4Model;

	let (width, height) = match lesson {
		Lesson::L2Triangle => {
			(200, 200)
		},
		Lesson::L1Model | Lesson::L2Model | Lesson::L3Model | Lesson::L4Model => {
			(800, 800)
		},
	};

	let mut image = tga::TgaImage::new(width as usize, height as usize, tga::TgaFormat::RGB);

	let mut model = model::Model::new();
	model.read(Path::new("african_head.obj")).unwrap();
	let texture = tga::TgaImage::read(Path::new("african_head_diffuse.tga")).unwrap();

	match lesson {
		Lesson::L1Model => {
			model.line(&mut image, 0, 0, width, height, white);
		},
		Lesson::L2Triangle => {
			image.fill(vec::Vec2::new(10, 70), vec::Vec2::new(50, 160), vec::Vec2::new(70, 80), red);
			image.fill(vec::Vec2::new(180, 50), vec::Vec2::new(150, 1), vec::Vec2::new(70, 180), white);
			image.fill(vec::Vec2::new(180, 150), vec::Vec2::new(120, 160), vec::Vec2::new(130, 180), green);
		},
		Lesson::L2Model => {
			model.fill(&mut image, 0, 0, width, height);
		},
		Lesson::L3Model => {
			let transform = &vec::position(0f64, 0f64, 0f64, width as f64, height as f64, 255f64);
			model.fill_float(&mut image, &texture, transform);
		},
		Lesson::L4Model => {
			let position = &vec::position(width as f64 / 8f64, height as f64 / 8f64, 0f64,
						      width as f64 * 0.75f64, height as f64 * 0.75f64, 255f64);
			let camera = &vec::project(3f64);
			let transform = &position.mul(camera);
			model.fill_float(&mut image, &texture, transform);
		},
	}

	let path = Path::new("output.tga");
	image.write(path, true).unwrap();

	/*
	let reader = File::open(path).unwrap();
	let reader = BufReader::new(reader);
	image::load(reader, image::ImageFormat::TGA).unwrap();
	*/
}
