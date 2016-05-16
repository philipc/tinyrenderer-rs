mod image;
mod model;
mod tga;
mod vec;

use std::{env, f64, path};

fn main() {
	let (width, height) = (800, 800);
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
	let light_transform = light.transform_vec(&transform).normalize();

	let mut image = image::Image::new(width, height, image::Format::Rgb);
	let mut zbuffer = vec![f64::MIN; image.get_width() * image.get_height()];
	for arg in env::args().skip(1) {
		let model = model::Model::read(path::Path::new(&format!("{}.obj", arg))).unwrap();
		let texture = Box::new(tga::read(path::Path::new(&format!("{}_diffuse.tga", arg))).unwrap());
		let normal = Box::new(tga::read(path::Path::new(&format!("{}_nm.tga", arg))).unwrap());
		let specular = Box::new(tga::read(path::Path::new(&format!("{}_spec.tga", arg))).unwrap());
		let mut shader = Shader {
			intensity: Intensity::NormalMapSpecular,
			color: Color::Texture,
			light: &light,
			light_transform: &light_transform,
			transform: &transform,
			transform_it: &transform_it,
			texture: texture,
			normal: normal,
			specular: specular,
			u: Default::default(),
			v: Default::default(),
			vert_intensity: Default::default(),
			vert_normal: Default::default(),
		};
		model.render(&mut image, &mut shader, viewport, &mut zbuffer[..]);
	}

	tga::write(&image, path::Path::new("output.tga"), true).unwrap();
}

struct Shader<'a> {
	// options
	intensity: Intensity,
	color: Color,

	// uniform
	light: &'a vec::Vec3<f64>,
	light_transform: &'a vec::Vec3<f64>,
	transform: &'a vec::Transform4<f64>,
	transform_it: &'a vec::Transform4<f64>,
	texture: Box<image::Image>,
	normal: Box<image::Image>,
	specular: Box<image::Image>,

	// varying
	u: vec::Vec3<f64>,
	v: vec::Vec3<f64>,
	vert_intensity: vec::Vec3<f64>,
	vert_normal: vec::Mat3<f64>,
}

enum Intensity {
	Gouraud,
	Phong,
	PhongTransform,
	PhongSpecular,
	NormalMap,
	NormalMapTransform,
	NormalMapSpecular,
}

enum Color {
	White,
	Texture,
}

impl<'a> image::Shader for Shader<'a> {
	fn vertex(&mut self, idx: usize, vert: &vec::Vec3<f64>, uv: &vec::Vec3<f64>, normal: &vec::Vec3<f64>) -> vec::Vec3<f64> {
		match self.intensity {
			Intensity::Gouraud => {
				self.vert_intensity.0[idx] = normal.dot(&self.light).max(0f64);
			},
			Intensity::Phong => {
				self.vert_normal.set_row(idx, normal);
			},
			Intensity::PhongTransform
			| Intensity::PhongSpecular => {
				self.vert_normal.set_row(idx, &normal.transform_vec(&self.transform_it));
			},
			Intensity::NormalMap
			| Intensity::NormalMapSpecular
			| Intensity::NormalMapTransform => { }
		}
		self.u.0[idx] = uv.0[0];
		self.v.0[idx] = uv.0[1];
		vert.transform_pt(&self.transform)
	}

	fn fragment(&self, bc: &vec::Vec3<f64>) -> Option<image::Color> {
		let u = (self.u.dot(bc) * self.texture.get_width() as f64).floor() as usize;
		let v = (self.v.dot(bc) * self.texture.get_height() as f64).floor() as usize;
		let ambient = 5f64;
		let mut diffuse = 0f64;
		let mut spec = 0f64;
		match self.intensity {
			Intensity::Gouraud => {
				diffuse = self.vert_intensity.dot(bc).max(0f64);
			},
			Intensity::Phong => {
				let normal = &self.vert_normal.dot_col(bc).normalize();
				diffuse = normal.dot(&self.light).max(0f64);
			},
			Intensity::PhongTransform => {
				let normal = &self.vert_normal.dot_col(bc).normalize();
				diffuse = normal.dot(&self.light_transform).max(0f64);
			},
			Intensity::PhongSpecular => {
				let normal = &self.vert_normal.dot_col(bc).normalize();
				diffuse = normal.dot(&self.light_transform).max(0f64);
				let reflect = normal.scale(2f64 * normal.dot(&self.light_transform)).sub(&self.light_transform).normalize();
				let spec_power = self.specular.get(u, v).r as i32;
				spec = reflect.0[2].max(0f64).powi(spec_power);
			},
			Intensity::NormalMap => {
				let normal = &self.normal.get(u, v).to_vec3f().normalize();
				diffuse = normal.dot(&self.light).max(0f64);
			},
			Intensity::NormalMapTransform => {
				let normal = self.normal.get(u, v).to_vec3f().transform_vec(&self.transform_it).normalize();
				diffuse = normal.dot(&self.light_transform).max(0f64);
			},
			Intensity::NormalMapSpecular => {
				let normal = self.normal.get(u, v).to_vec3f().transform_vec(&self.transform_it).normalize();
				diffuse = normal.dot(&self.light_transform).max(0f64);
				let reflect = normal.scale(2f64 * normal.dot(&self.light_transform)).sub(&self.light_transform).normalize();
				let spec_power = self.specular.get(u, v).r as i32 + 1;
				spec = reflect.0[2].max(0f64).powi(spec_power);
			},
		};
		let color = match self.color {
			Color::White => image::Color::new(255, 255, 255, 255),
			Color::Texture => self.texture.get(u, v),
		};
		let convert = |x| {
			(ambient + (x as f64) * (diffuse + 0.6 * spec)).min(255f64) as u8
		};
		Some(color.map(convert))
	}
}
