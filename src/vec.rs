use std::ops;

pub struct Vec3<T> {
	pub x: T,
	pub y: T,
	pub z: T,
}

impl<T> Vec3<T> where T: Copy + ops::Add<T, Output = T> + ops::Sub<T, Output = T> + ops::Mul<T, Output = T> + ops::Div<T, Output = T> {
	pub fn new(x: T, y: T, z: T) -> Self {
		Vec3 {
			x: x,
			y: y,
			z: z,
		}
	}

	pub fn sub(&self, v: &Vec3<T>) -> Self {
		Vec3 {
			x: self.x - v.x,
			y: self.y - v.y,
			z: self.z - v.z,
		}
	}

	pub fn dot(&self, v: &Vec3<T>) -> T {
		self.x * v.x + self.y * v.y + self.z * v.z
	}

	pub fn cross(&self, v: &Vec3<T>) -> Self {
		Vec3 {
			x: self.y * v.z - self.z * v.y,
			y: self.z * v.x - self.x * v.z,
			z: self.x * v.y - self.y * v.x,
		}
	}
}

impl Vec3<f64> {
	pub fn norm(&self) -> f64 {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	pub fn normalize(&mut self) -> Self {
		let n = self.norm();
		Vec3 {
			x: self.x / n,
			y: self.y / n,
			z: self.z / n,
		}
	}
}


