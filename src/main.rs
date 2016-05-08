mod image;
mod model;
mod tga;
mod vec;

use std::path;
use image::Shader;

fn main() {
	let (width, height) = (800, 800);
	let mut image = image::Image::new(width, height, image::Format::Rgb);

	let model = model::Model::read(path::Path::new("african_head.obj")).unwrap();
	let texture = tga::read(path::Path::new("african_head_diffuse.tga")).unwrap();

	let viewport = &vec::viewport(width as f64 / 8f64, height as f64 / 8f64, 0f64,
				      width as f64 * 0.75f64, height as f64 * 0.75f64, 255f64);
	let eye = &vec::Vec3([ 1f64, 1f64, 3f64 ]);
	let center = &vec::Vec3([ 0f64, 0f64, 0f64 ]);
	let up = &vec::Vec3([ 0f64, 1f64, 0f64 ]);
	let projection = &vec::project(eye.sub(center).norm());
	let modelview = &vec::lookat(eye, center, up);
	let transform = viewport.mul(projection).mul(modelview);
	let mut shader = MyShader {
		light: vec::Vec3::new(1f64, -1f64, 1f64).normalize(),
		transform: transform,
		.. Default::default()
	};
	model.render(&mut image, &mut shader, &texture);

	tga::write(&image, path::Path::new("output.tga"), true).unwrap();
}

#[derive(Default)]
struct MyShader {
	// uniform
	light: vec::Vec3<f64>,
	transform: vec::Transform4<f64>,

	// varying
	u: vec::Vec3<f64>,
	v: vec::Vec3<f64>,
	intensity: vec::Vec3<f64>,
}

impl Shader for MyShader {
	fn vertex(&mut self, idx: usize, vert: &vec::Vec3<f64>, uv: &vec::Vec3<f64>, normal: &vec::Vec3<f64>) -> vec::Vec3<f64> {
		self.u.0[idx] = uv.0[0];
		self.v.0[idx] = uv.0[1];
		self.intensity.0[idx] = normal.dot(&self.light);
		vert.transform(&self.transform)
	}

	fn fragment(&self, bc: &vec::Vec3<f64>, texture: &image::Image) -> Option<image::Color> {
		let texture_x = (self.u.dot(bc) * texture.get_width() as f64).floor() as usize;
		let texture_y = (self.v.dot(bc) * texture.get_height() as f64).floor() as usize;
		let intensity = self.intensity.dot(bc).max(0f64);
		let color = texture.get(texture_x, texture_y).intensity(intensity);
		Some(color)
	}
}
