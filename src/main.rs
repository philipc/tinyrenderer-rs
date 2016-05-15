mod image;
mod model;
mod tga;
mod vec;

use std::path;

fn main() {
	let (width, height) = (800, 800);
	let mut image = image::Image::new(width, height, image::Format::Rgb);

	let model = model::Model::read(path::Path::new("african_head.obj")).unwrap();
	let texture = Box::new(tga::read(path::Path::new("african_head_diffuse.tga")).unwrap());
	let normal = Box::new(tga::read(path::Path::new("african_head_nm.tga")).unwrap());

	let viewport = &vec::viewport(width as f64 / 8f64, height as f64 / 8f64, 0f64,
				      width as f64 * 0.75f64, height as f64 * 0.75f64, 255f64);
	let light = vec::Vec3([ 1f64, 1f64, 1f64 ]).normalize();
	let eye = &vec::Vec3([ 1f64, 1f64, 3f64 ]);
	let center = &vec::Vec3([ 0f64, 0f64, 0f64 ]);
	let up = &vec::Vec3([ 0f64, 1f64, 0f64 ]);
	let projection = &vec::project(eye, center);
	let modelview = &vec::lookat(eye, center, up);
	let transform = projection.mul(modelview);
	let transform_it = transform.inverse_transpose();
	//let light = light.transform_vec(&transform).normalize();
	let mut shader = Shader {
		light: light,
		transform: transform,
		transform_it: transform_it,
		texture: texture,
		normal: normal,
		.. Default::default()
	};
	model.render(&mut image, &mut shader, viewport);

	tga::write(&image, path::Path::new("output.tga"), true).unwrap();
}

#[derive(Default)]
struct Shader {
	// uniform
	light: vec::Vec3<f64>,
	transform: vec::Transform4<f64>,
	transform_it: vec::Transform4<f64>,
	texture: Box<image::Image>,
	normal: Box<image::Image>,

	// varying
	u: vec::Vec3<f64>,
	v: vec::Vec3<f64>,
	//intensity: vec::Vec3<f64>,
	//vnormal: vec::Mat3<f64>,
}

impl image::Shader for Shader {
	fn vertex(&mut self, idx: usize, vert: &vec::Vec3<f64>, uv: &vec::Vec3<f64>, normal: &vec::Vec3<f64>) -> vec::Vec3<f64> {
		self.u.0[idx] = uv.0[0];
		self.v.0[idx] = uv.0[1];
		//self.intensity.0[idx] = normal.dot(&self.light).max(0f64);
		//self.vnormal.set_row(idx, normal);
		//self.vnormal.set_row(idx, &normal.transform_vec(&self.transform_it));
		vert.transform_pt(&self.transform)
	}

	fn fragment(&self, bc: &vec::Vec3<f64>) -> Option<image::Color> {
		let u = (self.u.dot(bc) * self.texture.get_width() as f64).floor() as usize;
		let v = (self.v.dot(bc) * self.texture.get_height() as f64).floor() as usize;
		let normal = &self.normal.get(u, v).to_vec3f().normalize();
		//let normal = self.normal.get(u, v).to_vec3f().transform_vec(&self.transform_it).normalize();
		//let normal = &self.normal.dot_col(bc).transform_vec(&self.transform_it).normalize();
		//let normal = &self.vnormal.dot_col(bc).normalize();
		let intensity = normal.dot(&self.light).max(0f64);
		//let intensity = self.intensity.dot(bc).max(0f64);
		//let color = image::Color::new(255, 255, 255, 255).intensity(intensity);
		let color = self.texture.get(u, v).intensity(intensity);
		Some(color)
	}
}
