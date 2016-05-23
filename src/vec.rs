extern crate vecmath;

use std::ops;

#[derive(Debug)]
pub struct Vec2<T> (pub vecmath::Vector2<T>);

impl<T> Vec2<T> where T: Copy {
	pub fn new(x: T, y: T) -> Self {
		Vec2([x, y])
	}

	pub fn as_tuple(&self) -> (T, T) {
		(self.0[0], self.0[1])
	}
}

#[derive(Debug)]
pub struct Vec3<T> (pub vecmath::Vector3<T>);

impl Default for Vec3<f64> {
	fn default() -> Self {
		Vec3([0f64, 0f64, 0f64])
	}
}

impl<T> Vec3<T> where T: Copy {
	pub fn new(x: T, y: T, z: T) -> Self {
		Vec3([x, y, z])
	}

	pub fn as_tuple(&self) -> (T, T, T) {
		(self.0[0], self.0[1], self.0[2])
	}
}

impl<T> Vec3<T> where T: Copy + ops::Add<T, Output = T> + ops::Sub<T, Output = T> + ops::Mul<T, Output = T> + ops::Div<T, Output = T> {
	pub fn scale(&self, n: T) -> Self {
		Vec3(vecmath::vec3_scale(self.0, n))
	}

	pub fn sub(&self, v: &Vec3<T>) -> Self {
		Vec3(vecmath::vec3_sub(self.0, v.0))
	}

	pub fn dot(&self, v: &Vec3<T>) -> T {
		vecmath::vec3_dot(self.0, v.0)
	}

	pub fn cross(&self, v: &Vec3<T>) -> Self {
		Vec3(vecmath::vec3_cross(self.0, v.0))
	}
}

impl Vec3<f64> {
	pub fn norm(&self) -> f64 {
		(self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]).sqrt()
	}

	pub fn normalize(&mut self) -> Self {
		let n = self.norm();
		Vec3([
		     self.0[0] / n,
		     self.0[1] / n,
		     self.0[2] / n ])
	}

	pub fn to_pt4(&self) -> Vec4<f64> {
		Vec4([
		     self.0[0],
		     self.0[1],
		     self.0[2],
		     1f64 ])
	}

	pub fn to_vec4(&self) -> Vec4<f64> {
		Vec4([
		     self.0[0],
		     self.0[1],
		     self.0[2],
		     0f64 ])
	}

	pub fn transform_pt(&self, transform: &Transform4<f64>) -> Self {
		self.to_pt4().transform(transform).to_pt3()
	}

	pub fn transform_vec(&self, transform: &Transform4<f64>) -> Self {
		self.to_vec4().transform(transform).to_vec3()
	}

	pub fn mul(&self, mat: &Mat3<f64>) -> Self {
		Vec3(vecmath::row_mat3_transform(mat.0, self.0))
	}
}

#[derive(Debug)]
pub struct Vec4<T> (pub vecmath::Vector4<T>);

impl Vec4<f64> {
	pub fn to_pt3(&self) -> Vec3<f64> {
		let w = self.0[3];
		Vec3([
		     self.0[0] / w,
		     self.0[1] / w,
		     self.0[2] / w ])
	}

	pub fn to_vec3(&self) -> Vec3<f64> {
		Vec3([
		     self.0[0],
		     self.0[1],
		     self.0[2] ])
	}

	pub fn transform(&self, transform: &Transform4<f64>) -> Self {
		Vec4(vecmath::row_mat4_transform(transform.0, self.0))
	}
}

#[derive(Debug)]
pub struct Mat3<T> (pub vecmath::Matrix3<T>);

impl Default for Mat3<f64> {
	fn default() -> Self {
		Mat3(vecmath::mat3_id())
	}
}

impl Mat3<f64> {
	pub fn set_row(&mut self, i: usize, v: &Vec3<f64>) {
		self.0[i] = v.0;
	}

	pub fn set_col(&mut self, i: usize, v: &Vec3<f64>) {
		self.0[0][i] = v.0[0];
		self.0[1][i] = v.0[1];
		self.0[2][i] = v.0[2];
	}

	pub fn interpolate(&self, v: &Vec3<f64>) -> Vec3<f64> {
		Vec3(vecmath::col_mat3_transform(self.0, v.0))
	}

	pub fn inv(&self) -> Self {
		Mat3(vecmath::mat3_inv(self.0))
	}
}

#[derive(Debug)]
pub struct Transform4<T> (pub vecmath::Matrix4<T>);

impl Default for Transform4<f64> {
	fn default() -> Self {
		Transform4(vecmath::mat4_id())
	}
}

impl Transform4<f64> {
	pub fn mul(&self, mat: &Transform4<f64>) -> Self {
		Transform4(vecmath::row_mat4_mul(self.0, mat.0))
	}

	pub fn inverse_transpose(&self) -> Self {
		Transform4(vecmath::mat4_inv(vecmath::mat4_transposed(self.0)))
	}
}

pub fn viewport(x: f64, y: f64, z: f64, w: f64, h: f64, d: f64) -> Transform4<f64> {
	let mut mat = vecmath::mat4_id();
	mat[0][3] = x + w / 2f64;
	mat[1][3] = y + h / 2f64;
	mat[2][3] = z + d / 2f64;
	mat[0][0] = w / 2f64;
	mat[1][1] = h / 2f64;
	mat[2][2] = d / 2f64;
	Transform4(mat)
}

pub fn project(eye: &Vec3<f64>, center: &Vec3<f64>) -> Transform4<f64> {
	let mut mat = vecmath::mat4_id();
	mat[3][2] = -1f64 / eye.sub(center).norm();
	Transform4(mat)
}

pub fn lookat(eye: &Vec3<f64>, center: &Vec3<f64>, up: &Vec3<f64>) -> Transform4<f64> {
	let mut translate = vecmath::mat4_id();
	translate[0][3] = -center.0[0];
	translate[1][3] = -center.0[1];
	translate[2][3] = -center.0[2];

	let z = &eye.sub(center).normalize();
	let x = &up.cross(z).normalize();
	let y = &z.cross(x).normalize();
	let mut rotate = vecmath::mat4_id();
	for i in 0..3 {
		rotate[0][i] = x.0[i];
		rotate[1][i] = y.0[i];
		rotate[2][i] = z.0[i];
	}
	Transform4(rotate).mul(&Transform4(translate))
}
