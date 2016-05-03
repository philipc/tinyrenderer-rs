extern crate vecmath;

use std::ops;

pub struct Vec2<T> (pub vecmath::Vector2<T>);

impl<T> Vec2<T> where T: Copy {
	pub fn new(x: T, y: T) -> Self {
		Vec2([x, y])
	}

	#[allow(dead_code)]
        pub fn as_tuple(&self) -> (T, T) {
                (self.0[0], self.0[1])
        }
}

pub struct Vec3<T> (pub vecmath::Vector3<T>);

impl<T> Vec3<T> where T: Copy {
	pub fn new(x: T, y: T, z: T) -> Self {
		Vec3([x, y, z])
	}

        pub fn as_tuple(&self) -> (T, T, T) {
                (self.0[0], self.0[1], self.0[2])
        }
}

impl<T> Vec3<T> where T: Copy + ops::Add<T, Output = T> + ops::Sub<T, Output = T> + ops::Mul<T, Output = T> + ops::Div<T, Output = T> {
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
}
