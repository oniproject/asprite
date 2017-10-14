use std::ops::{Add, Sub, AddAssign, SubAssign};

pub mod stagger {
	pub enum Index {
		Odd, Even,
	}
	pub enum Axis {
		Q, R,
	}
	pub enum Combo {
		RQ,
		QS,
		SR,
		QR,
		SQ,
		RS,
	}
}

struct FHex {
	q: f32,
	r: f32,
	s: f32,
}

impl FHex {
	fn round(&self) -> (f32, f32, f32) {
		let q = self.q.round();
		let r = self.r.round();
		let s = self.s.round();
		let q_diff = (self.q - q).abs();
		let r_diff = (self.r - r).abs();
		let s_diff = (self.s - s).abs();
		if q_diff > r_diff && q_diff > s_diff {
			(-r - s, r, s)
		} else if r_diff > s_diff {
			(q, -q - s, s)
		} else {
			(q, r, -q - r)
		}
	}
}

pub trait HexCoord {
	fn q(&self);
	fn r(&self);
	fn s(&self);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cube {
	pub x: isize,
	pub y: isize,
	pub z: isize,
}

pub fn length(q: isize, r: isize, s: isize) -> isize {
	(q.abs() + r.abs() + s.abs()) / 2
}

/*
int hex_distance(Hex a, Hex b) {
	return hex_length(hex_subtract(a, b));
}
*/

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Axial {
	pub q: isize,
	pub r: isize,
}

impl Axial {
	pub fn to_cube(self) -> Cube { 
		let x = self.q;
		let z = self.r;
		let y = -x - z;
		Cube { x, y, z }
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hex {
	pub x: isize,
	pub y: isize,
	pub z: isize,
}

impl Hex {
	pub fn new(x: isize, y: isize, z: isize) -> Self {
		Self { x, y, z }
	}
	pub fn staggered(col: isize, row: isize, index: stagger::Index, axis: stagger::Axis) -> Self {
		use self::stagger::{Axis, Index};
		let (x, z) = match (axis, index) {
			(Axis::Q, Index::Even) => (col, row - (col + (col & 1)) / 2),
			(Axis::Q, Index::Odd)  => (col, row - (col - (col & 1)) / 2),
			(Axis::R, Index::Even) => (col - (row + (row & 1)) / 2, row),
			(Axis::R, Index::Odd)  => (col - (row - (row & 1)) / 2, row),
		};
		let y = -x - z;
		Self { x, y, z }
	}

	pub fn to_staggered(&self, index: stagger::Index, axis: stagger::Axis) -> (isize, isize) {
		use self::stagger::{Axis, Index};
		match (axis, index) {
			(Axis::Q, Index::Even) => (self.x, self.z + (self.x + (self.x & 1)) / 2),
			(Axis::Q, Index::Odd)  => (self.x, self.z + (self.x - (self.x & 1)) / 2),
			(Axis::R, Index::Even) => (self.x + (self.z + (self.z & 1)) / 2, self.z),
			(Axis::R, Index::Odd)  => (self.x + (self.z - (self.z & 1)) / 2, self.z),
		}
	}

	pub fn rotate_left(&mut self) {
		let x = self.x;
		self.x = -self.y;
		self.y = -self.z;
		self.z = -x;
	}

	pub fn rotate_right(&mut self) {
		let x = self.x;
		self.x = -self.z;
		self.y = -self.y;
		self.z = -x;
	}
}

impl Add for Hex {
	type Output = Hex;
	fn add(self, other: Self) -> Self {
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
}

impl Sub for Hex {
	type Output = Hex;
	fn sub(self, other: Self) -> Self {
		Self {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
		}
	}
}

impl AddAssign for Hex {
	fn add_assign(&mut self, other: Self) {
		*self = *self + other;
	}
}

impl SubAssign for Hex {
	fn sub_assign(&mut self, other: Self) {
		*self = *self - other;
	}
}