mod image;
mod model;
mod tga;
mod vec;

use std::{env, f64, path};

fn main() {
	let (width, height) = (800, 800);
	let viewport = &vec::viewport(width as f64 / 8f64, height as f64 / 8f64, 0f64,
				      width as f64 * 0.75f64, height as f64 * 0.75f64, 255f64);
	let light = &vec::Vec3([ 1f64, 1f64, 0f64 ]).normalize();
	let eye = &vec::Vec3([ 1f64, 1f64, 4f64 ]);
	let center = &vec::Vec3([ 0f64, 0f64, 0f64 ]);
	let up = &vec::Vec3([ 0f64, 1f64, 0f64 ]);

	let projection = &vec::project(eye, center);
	let modelview = &vec::lookat(eye, center, up);
	let transform = projection.mul(modelview);
	let transform_it = transform.inverse_transpose();
	let light_transform = light.transform_vec(&transform).normalize();

	let (shadow_width, shadow_height) = (800, 800);
	let shadow_viewport = &vec::viewport(shadow_width as f64 / 8f64, shadow_height as f64 / 8f64, 0f64,
					     shadow_width as f64 * 0.75f64, shadow_height as f64 * 0.75f64, 1f64);
	let shadow_transform = &vec::lookat(light, center, up);

	let mut image = image::Image::new(width, height, image::Format::Rgb);
	let mut shadow_image = image::Image::new(shadow_width, shadow_height, image::Format::Rgb);
	let mut zbuffer = vec![f64::MIN; width * height];
	let mut shadow_zbuffer = vec![f64::MIN; shadow_width * shadow_height];
	for arg in env::args().skip(1) {
		let model = model::Model::read(path::Path::new(&format!("{}.obj", arg))).unwrap();
		let texture = Box::new(tga::read(path::Path::new(&format!("{}_diffuse.tga", arg))).unwrap());
		//let texture = Box::new(tga::read(path::Path::new("obj/grid.tga")).unwrap());
		let normal = Box::new(tga::read(path::Path::new(&format!("{}_nm.tga", arg))).unwrap_or(image::Image::default()));
		let tangent = Box::new(tga::read(path::Path::new(&format!("{}_nm_tangent.tga", arg))).unwrap_or(image::Image::default()));
		let specular = Box::new(tga::read(path::Path::new(&format!("{}_spec.tga", arg))).unwrap_or(image::Image::default()));

		let shadow = false;
		if shadow {
			let mut shadow_shader = ShadowShader {
				shadow_transform: &shadow_transform,
				shadow_viewport: shadow_viewport,
				shadow_vert: Default::default(),
			};
			model.render(&mut shadow_image, &mut shadow_shader, shadow_viewport, &mut shadow_zbuffer[..]);
		}

		let mut shader = Shader {
			intensity: Intensity::Constant,
			color: Color::Texture,
			shadow: shadow,

			light: light,
			light_transform: &light_transform,
			transform: &transform,
			transform_it: &transform_it,

			texture: texture,
			normal: normal,
			specular: specular,
			tangent: tangent,

			shadow_transform: &shadow_transform,
			shadow_zbuffer: &shadow_zbuffer,
			shadow_width: shadow_width,
			shadow_height: shadow_height,
			shadow_viewport: shadow_viewport,

			u: Default::default(),
			v: Default::default(),
			vert: Default::default(),
			shadow_vert: Default::default(),
			vert_intensity: Default::default(),
			vert_normal: Default::default(),
		};
		model.render(&mut image, &mut shader, viewport, &mut zbuffer[..]);
	}

	tga::write(&image, path::Path::new("output.tga"), true).unwrap();
	tga::write(&shadow_image, path::Path::new("shadow.tga"), true).unwrap();
}

struct ShadowShader<'a> {
	// uniform
	shadow_transform: &'a vec::Transform4<f64>,
	shadow_viewport: &'a vec::Transform4<f64>,

	// varying
	shadow_vert: vec::Mat3<f64>,
}

impl<'a> image::Shader for ShadowShader<'a> {
	fn vertex(&mut self, idx: usize, vert: &vec::Vec3<f64>, uv: &vec::Vec3<f64>, normal: &vec::Vec3<f64>) -> vec::Vec4<f64> {
		self.shadow_vert.set_row(idx, &vert.transform_pt(&self.shadow_transform));
		vert.to_pt4().transform(&self.shadow_transform)
	}

	fn fragment(&self, bc: &vec::Vec3<f64>) -> Option<image::Color> {
		let shadow_p = self.shadow_vert.dot_col(bc).transform_pt(self.shadow_viewport);
		Some(image::Color::new(255, 255, 255, 255).intensity(shadow_p.0[2]))
	}
}

struct Shader<'a> {
	// options
	intensity: Intensity,
	color: Color,
	shadow: bool,

	// uniform
	light: &'a vec::Vec3<f64>,
	light_transform: &'a vec::Vec3<f64>,
	transform: &'a vec::Transform4<f64>,
	transform_it: &'a vec::Transform4<f64>,
	texture: Box<image::Image>,
	normal: Box<image::Image>,
	tangent: Box<image::Image>,
	specular: Box<image::Image>,

	// shadow
	shadow_transform: &'a vec::Transform4<f64>,
	shadow_zbuffer: &'a [f64],
	shadow_width: usize,
	shadow_height: usize,
	shadow_viewport: &'a vec::Transform4<f64>,

	// varying
	u: vec::Vec3<f64>,
	v: vec::Vec3<f64>,
	vert: [vec::Vec3<f64>; 3],
	shadow_vert: vec::Mat3<f64>,
	vert_intensity: vec::Vec3<f64>,
	vert_normal: vec::Mat3<f64>,
}

enum Intensity {
	Constant,
	Gouraud,
	Phong,
	PhongTransform,
	PhongSpecular,
	NormalMap,
	NormalMapTransform,
	NormalMapSpecular,
	TangentMap,
}

enum Color {
	White,
	Texture,
}

impl<'a> image::Shader for Shader<'a> {
	fn vertex(&mut self, idx: usize, vert: &vec::Vec3<f64>, uv: &vec::Vec3<f64>, normal: &vec::Vec3<f64>) -> vec::Vec4<f64> {
		match self.intensity {
			Intensity::Gouraud
			=> {
				self.vert_intensity.0[idx] = normal.dot(&self.light).max(0f64);
			},
			Intensity::Phong
			=> {
				self.vert_normal.set_row(idx, normal);
			},
			Intensity::PhongTransform
			| Intensity::PhongSpecular
			| Intensity::TangentMap
			=> {
				self.vert_normal.set_row(idx, &normal.transform_vec(&self.transform_it));
			},
			Intensity::Constant
			| Intensity::NormalMap
			| Intensity::NormalMapSpecular
			| Intensity::NormalMapTransform
			=> { }
		}
		self.u.0[idx] = uv.0[0];
		self.v.0[idx] = uv.0[1];
		self.vert[idx].0 = vert.transform_pt(&self.transform).0;
		if self.shadow {
			self.shadow_vert.set_row(idx, &vert.transform_pt(&self.shadow_transform));
		}
		vert.to_pt4().transform(&self.transform)
	}

	fn fragment(&self, bc: &vec::Vec3<f64>) -> Option<image::Color> {
		let u = (self.u.dot(bc) * self.texture.get_width() as f64).floor() as usize;
		let v = (self.v.dot(bc) * self.texture.get_height() as f64).floor() as usize;
		let ambient = 0f64;
		let mut diffuse = 0f64;
		let mut spec = 0f64;
		match self.intensity {
			Intensity::Constant => {
				diffuse = 1.0;
			},
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
			Intensity::TangentMap => {
				let n = &self.vert_normal.dot_col(bc).normalize();

				// Three vectors for which we know the dot product with u/v/n
				let a = &mut vec::Mat3::default();
				a.set_row(0, &self.vert[1].sub(&self.vert[0]));
				a.set_row(1, &self.vert[2].sub(&self.vert[0]));
				a.set_row(2, n);
				let ai = &a.inv();

				// Solve for u/v/n and normalize
				let b = &mut vec::Mat3::default();
				b.set_col(0, &vec::Vec3([ self.u.0[1] - self.u.0[0], self.u.0[2] - self.u.0[0], 0f64]).mul(ai).normalize());
				b.set_col(1, &vec::Vec3([ self.v.0[1] - self.v.0[0], self.v.0[2] - self.v.0[0], 0f64]).mul(ai).normalize());
				b.set_col(2, n);

				let normal = &self.tangent.get(u, v).to_vec3f().mul(b).normalize();
				diffuse = normal.dot(&self.light_transform).max(0f64);
			},
		};
		let mut shadow = 1.0;
		if self.shadow {
			let shadow_p = self.shadow_vert.dot_col(bc).transform_pt(self.shadow_viewport);
			let shadow_x = shadow_p.0[0] as usize;
			let shadow_y = shadow_p.0[1] as usize;
			if shadow_x < self.shadow_width && shadow_y < self.shadow_height {
				let shadow_z = self.shadow_zbuffer[shadow_x + shadow_y * self.shadow_width];
				if shadow_p.0[2] + 0.01 < shadow_z {
					shadow = 0.3;
				}
			}
		}
		let color = match self.color {
			Color::White => image::Color::new(255, 255, 255, 255),
			Color::Texture => self.texture.get(u, v),
		};
		let convert = |x| {
			(ambient + (x as f64) * shadow * (diffuse + 0.6 * spec)).min(255f64) as u8
		};
		Some(color.map(convert))
	}
}
