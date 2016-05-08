mod image;
mod model;
mod tga;
mod vec;

use std::path;

fn main() {
	let (width, height) = (800, 800);
	let mut image = image::Image::new(width, height, image::Format::RGB);

	let model = model::Model::read(path::Path::new("african_head.obj")).unwrap();
	let texture = tga::read(path::Path::new("african_head_diffuse.tga")).unwrap();

	let viewport = &vec::viewport(width as f64 / 8f64, height as f64 / 8f64, 0f64,
				      width as f64 * 0.75f64, height as f64 * 0.75f64, 255f64);
	let eye = &vec::Vec3([ 1f64, 1f64, 3f64 ]);
	let center = &vec::Vec3([ 0f64, 0f64, 0f64 ]);
	let up = &vec::Vec3([ 0f64, 1f64, 0f64 ]);
	let projection = &vec::project(eye.sub(center).norm());
	let modelview = &vec::lookat(eye, center, up);
	let transform = &viewport.mul(projection).mul(modelview);
	model.render(&mut image, &texture, transform);

	tga::write(&image, path::Path::new("output.tga"), true).unwrap();
}
