use super::*;

use std::mem::swap;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Rect<T> {
	pub min: Point2<T>,
	pub max: Point2<T>,
}

impl<S> Default for Rect<S>
	where S: BaseNum
{
	fn default() -> Self {
		Self {
			min: Point2::new(S::zero(), S::zero()),
			max: Point2::new(S::zero(), S::zero()),
		}
	}
}

impl<S> Rect<S> where S: BaseNum {
	pub fn dx(&self) -> S { self.max.x - self.min.x }
	pub fn dy(&self) -> S { self.max.y - self.min.y }
	pub fn dim(&self) -> Vector2<S> { Vector2::new(self.dx(), self.dy()) }
}

impl<S> Rect<S> where S: BaseNum {
	pub fn contains(&self, p: Point2<S>) -> bool {
		self.min.x <= p.x && p.x <= self.max.x &&
		self.min.y <= p.y && p.y <= self.max.y
	}

	pub fn contains_xy(&self, x: S, y: S) -> bool {
		self.min.x <= x && x <= self.max.x &&
		self.min.y <= y && y <= self.max.y
	}

	pub fn contains_rect(&self, r: &Self) -> bool {
		self.contains(r.min) && self.contains(r.max)
	}

	#[inline(always)]
	fn is_empty(&self) -> bool {
		self.min.x >= self.max.x || self.min.y >= self.max.y
	}

	#[inline(always)]
	fn min(a: S, b: S) -> S {
		if a < b { a } else { b }
	}

	#[inline(always)]
	fn max(a: S, b: S) -> S {
		if a > b { a } else { b }
	}

	pub fn intersect(mut self, s: Self) -> Option<Self> {
		self.min.x = Self::max(self.min.x, s.min.x);
		self.min.y = Self::max(self.min.y, s.min.y);
		self.max.x = Self::min(self.max.x, s.max.x);
		self.max.y = Self::min(self.max.y, s.max.y);
		if self.is_empty() {
			None
		} else {
			Some(self)
		}
	}

	pub fn union_with_empty(self, s: Self) -> Self {
		if self.is_empty() {
			s
		} else if s.is_empty() {
			self
		} else {
			Self {
				min: Point2 {
					x: Self::min(self.min.x, s.min.x),
					y: Self::min(self.min.y, s.min.y),
				},
				max: Point2 {
					x: Self::max(self.max.x, s.max.x),
					y: Self::max(self.max.y, s.max.y),
				}
			}
		}
	}

	pub fn union(mut self, s: Self) -> Option<Self> {
		if self.is_empty() || s.is_empty() {
			None
		} else {
			self.min.x = Self::min(self.min.x, s.min.x);
			self.min.y = Self::min(self.min.y, s.min.y);
			self.max.x = Self::max(self.max.x, s.max.x);
			self.max.y = Self::max(self.max.y, s.max.y);
			Some(self)
		}
	}

	pub fn union_raw(mut self, s: Self) -> Self {
		self.min.x = Self::min(self.min.x, s.min.x);
		self.min.y = Self::min(self.min.y, s.min.y);
		self.max.x = Self::max(self.max.x, s.max.x);
		self.max.y = Self::max(self.max.y, s.max.y);
		self
	}

	pub fn union_point(self, p: Point2<S>) -> Self {
		self.union_xy(p.x, p.y)
	}

	pub fn union_xy(mut self, x: S, y: S) -> Self {
		self.min.x = Self::min(self.min.x, x);
		self.min.y = Self::min(self.min.y, y);
		self.max.x = Self::max(self.max.x, x);
		self.max.y = Self::max(self.max.y, y);
		self
	}

	pub fn normalize(self) -> Self {
		let Rect { mut min, mut max } = self;
		if min.x > max.x {
			swap(&mut min.x, &mut max.x);
		}
		if min.y > max.y {
			swap(&mut min.y, &mut max.y);
		}
		Self { min, max }
	}
}

// constructor
impl<S> Rect<S> where S: BaseNum {
	pub fn from_min_dim(min: Point2<S>, dim: Vector2<S>) -> Self {
		Self { min, max: min + dim }
	}
	pub fn from_min_max(min: Point2<S>, max: Point2<S>) -> Self {
		Self { min, max }
	}
	pub fn from_coords(x1: S, y1: S, x2: S, y2: S) -> Self {
		Self {
			min: Point2::new(x1, y1),
			max: Point2::new(x2, y2),
		}
	}
}

// builder
impl<S> Rect<S> where S: BaseNum {
	pub fn shift_x(self, x: S) -> Self {
		Self {
			min: Point2::new(self.min.x + x, self.min.y),
			max: Point2::new(self.min.x + x, self.min.y),
		}
	}
	pub fn shift_y(self, y: S) -> Self {
		Self {
			min: Point2::new(self.min.x, self.min.y + y),
			max: Point2::new(self.min.x, self.min.y + y),
		}
	}
	pub fn shift_x_y(self, x: S, y: S) -> Self {
		Self {
			min: Point2::new(self.min.x + x, self.min.y + y),
			max: Point2::new(self.min.x + x, self.min.y + y),
		}
	}

	pub fn shift_xy(self, xy: Vector2<S>) -> Self {
		Self {
			min: self.min + xy,
			max: self.max + xy,
		}
	}

	pub fn pad_min_x(self, pad: S) -> Self { Self { min: Point2::new(self.min.x + pad, self.min.y), .. self } }
	pub fn pad_max_x(self, pad: S) -> Self { Self { max: Point2::new(self.max.x - pad, self.max.y), .. self } }

	pub fn pad_min_y(self, pad: S) -> Self { Self { min: Point2::new(self.min.x, self.min.y + pad), .. self } }
	pub fn pad_max_y(self, pad: S) -> Self { Self { max: Point2::new(self.max.x, self.max.y - pad), .. self } }

	pub fn pad(self, pad: S) -> Self {
		Self {
			min: Point2::new(self.min.x + pad, self.min.y + pad),
			max: Point2::new(self.max.x - pad, self.max.y - pad),
		}
	}

	pub fn pad_x(self, pad: S) -> Self {
		Self {
			min: Point2::new(self.min.x + pad, self.min.y),
			max: Point2::new(self.max.x - pad, self.max.y),
		}
	}

	pub fn pad_y(self, pad: S) -> Self {
		Self {
			min: Point2::new(self.min.x, self.min.y + pad),
			max: Point2::new(self.max.x, self.max.y - pad),
		}
	}
}

impl<S> Rect<S> where S: BaseFloat {
	#[inline(always)]
	fn lerp(min: S, max: S, t: S) -> S {
		(S::one() - t) * min + t * max
	}

	pub fn split_y(self, t: S) -> (Self, Self) {
		let Self { min, max } = self;
		let y = Self::lerp(min.y, max.y, t);
		(
			Self { min, max: Point2::new(max.x, y) },
			Self { max, min: Point2::new(min.x, y) },
		)
	}
	pub fn split_x(self, t: S) -> (Self, Self) {
		let Self { min, max } = self;
		let x = Self::lerp(min.x, max.x, t);
		(
			Self { min, max: Point2::new(x, max.y) },
			Self { max, min: Point2::new(x, min.y) },
		)
	}
}
