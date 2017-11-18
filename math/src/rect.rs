use super::*;
use cgmath::Vector1;

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

impl<S> Rect<S>
	where S: BaseNum
{
	pub fn new() -> Self {
		Self::default()
	}

	pub fn dx(&self) -> S { self.max.x - self.min.x }
	pub fn dy(&self) -> S { self.max.y - self.min.y }

	pub fn with_coords(x1: S, y1: S, x2: S, y2: S) -> Self {
		Self {
			min: Point2::new(x1, y1),
			max: Point2::new(x2, y2),
		}
	}
	pub fn with_size(x: S, y: S, w: S, h: S) -> Self {
		Self {
			min: Point2::new(x, y),
			max: Point2::new(x + w, y + h),
		}
	}

	pub fn xy(self, x: S, y: S) -> Self {
		let p = Vector2::new(x, y);
		Self {
			min: self.min + p,
			max: self.max + p,
		}
	}
	pub fn x(self, x: S) -> Self {
		let p = Vector2::new(x, S::zero());
		Self {
			min: self.min + p,
			max: self.max + p,
		}
	}

	pub fn y(self, y: S) -> Self {
		let p = Vector2::new(S::zero(), y);
		Self {
			min: self.min + p,
			max: self.max + p,
		}
	}

	pub fn w(self, w: S) -> Self {
		Self {
			min: self.min,
			max: self.min + Vector2::new(w, self.dy()),
		}
	}

	pub fn h(self, h: S) -> Self {
		Self {
			min: self.min,
			max: self.min + Vector2::new(self.dx(), h),
		}
	}

	pub fn wh(self, w: S, h: S) -> Self {
		Self {
			min: self.min,
			max: self.min + Vector2::new(w, h),
		}
	}
}

impl<S> Rect<S>
	where S: BaseNum
{
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

	pub fn inset(self, n: S) -> Self {
		self.inset_xy(n, n)
	}

	pub fn inset_xy(self, x: S, y: S) -> Self {
		let Self { mut min, mut max } = self;
		let two = S::one() + S::one();
		let dx = max.x - min.x;
		let dy = max.y - min.y;
		if dx < two*x {
			min.x = (min.x + max.x) / two;
			max.x = min.x;
		} else {
			min.x += x;
			max.x -= x;
		}
		if dy < two*y {
			min.y = (min.y + max.y) / two;
			max.y = min.y;
		} else {
			min.y += y;
			max.y -= y;
		}
		Self {
			min, max
		}
	}

	pub fn inset_x(self, n: S) -> Self {
		let Self { mut min, mut max } = self;
		let two = S::one() + S::one();
		let dx = max.x - min.x;
		if dx < two*n {
			min.x = (min.x + max.x) / two;
			max.x = min.x;
		} else {
			min.x += n;
			max.x -= n;
		}
		Self {
			min, max
		}
	}

	pub fn inset_y(self, n: S) -> Self {
		let Self { mut min, mut max } = self;
		let two = S::one() + S::one();
		let dy = max.y - min.y;
		if dy < two*n {
			min.y = (min.y + max.y) / two;
			max.y = min.y;
		} else {
			min.y += n;
			max.y -= n;
		}
		Self {
			min, max
		}
	}

	fn is_empty(&self) -> bool {
		self.min.x >= self.max.x || self.min.y >= self.max.y
	}

	#[inline(always)]
	fn min(a: S, b: S) -> S {
		if a < b {
			a
		} else {
			b
		}
	}

	#[inline(always)]
	fn max(a: S, b: S) -> S {
		if a > b {
			a
		} else {
			b
		}
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

	pub fn translate(&self, p: Vector2<S>) -> Self {
		let min = self.min + p;
		let max = self.max + p;
		Self { min, max }
	}

	pub fn min_translate(&self, p: Vector2<S>) -> Point2<S> {
		self.min + p
	}

	pub fn min_translate_rect(&self, r: Self) -> Self {
		let from = Vector2::new(self.min.x, self.min.y);
		let min = r.min + from;
		let max = r.max + from;
		Self { min, max }
	}

	pub fn normalize(self) -> Self {
		let Rect { mut min, mut max } = self;
		if min.x > max.x {
			swap(&mut min.x, &mut max.x);
		}
		if min.y > max.y {
			swap(&mut min.y, &mut max.y);
		}
		Self {
			min, max
		}
	}

	pub fn align<F: BaseFloat>(&self, x: F, y: F, size: Point2<S>) -> Point2<S> {
		let (tw, th) = (size.x, size.y);
		let (rw, rh) = (self.dx(), self.dy());

		let dw = F::from(rw - tw).unwrap();
		let dh = F::from(rh - th).unwrap();

		let x = self.min.x + S::from(dw * x).unwrap();
		let y = self.min.y + S::from(dh * y).unwrap();

		Point2::new(x, y)
	}
}


impl<S> Rect<S>
	where S: BaseFloat
{
	#[inline(always)]
	fn lerp(a: S, b: S, v: S) -> S {
		Vector1::new(a).lerp(Vector1::new(b), v).x
	}

	pub fn transform(self, anchor: Self, offset: Self) -> Self {
		Self {
			min: offset.min + Vector2::new(
				Self::lerp(self.min.x, self.max.x, anchor.min.x),
				Self::lerp(self.min.y, self.max.y, anchor.min.y),
			),
			max: offset.max + Vector2::new(
				Self::lerp(self.min.x, self.max.x, anchor.max.x),
				Self::lerp(self.min.y, self.max.y, anchor.max.y),
			),
		}
	}
}

#[test]
fn test() {
	let canvas = Rect {
		min: Point2::new(0.0, 0.0),
		max: Point2::new(100.0, 100.0),
	};

	let anchor = Rect {
		min: Point2::new(0.25, 0.25),
		max: Point2::new(0.75, 0.75),
	};

	println!("{:?}", canvas.transform(anchor, Rect {
		min: Point2::new(-10.0, -10.0),
		max: Point2::new(10.0, 10.0),
	}));
}