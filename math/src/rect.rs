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
	#[inline]
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
	#[inline]
	pub fn new() -> Self {
		Self::default()
	}

	#[inline]
	pub fn dx(&self) -> S { self.max.x - self.min.x }
	#[inline]
	pub fn dy(&self) -> S { self.max.y - self.min.y }

	#[inline]
	pub fn with_coords(x1: S, y1: S, x2: S, y2: S) -> Self {
		Self {
			min: Point2::new(x1, y1),
			max: Point2::new(x2, y2),
		}
	}
	#[inline]
	pub fn with_size(x: S, y: S, w: S, h: S) -> Self {
		Self {
			min: Point2::new(x, y),
			max: Point2::new(x + w, y + h),
		}
	}

	#[inline]
	pub fn x(self, x: S) -> Self {
		self.xy(x, S::zero())
	}

	#[inline]
	pub fn y(self, y: S) -> Self {
		self.xy(S::zero(), y)
	}

	#[inline]
	pub fn xy(self, x: S, y: S) -> Self {
		self.pos(Vector2::new(x, y))
	}

	#[inline]
	pub fn pos(self, p: Vector2<S>) -> Self {
		Self {
			min: self.min + p,
			max: self.max + p,
		}
	}

	#[inline]
	pub fn w(self, w: S) -> Self {
		self.wh(w, self.dy())
	}

	#[inline]
	pub fn h(self, h: S) -> Self {
		self.wh(self.dx(), h)
	}

	#[inline]
	pub fn wh(self, w: S, h: S) -> Self {
		self.dim(Vector2::new(w, h))
	}

	#[inline]
	pub fn dim(self, v: Vector2<S>) -> Self {
		Self {
			min: self.min,
			max: self.min + v,
		}
	}
}

impl<S> Rect<S>
	where S: BaseNum
{
	#[inline]
	pub fn contains(&self, p: Point2<S>) -> bool {
		self.min.x <= p.x && p.x <= self.max.x &&
		self.min.y <= p.y && p.y <= self.max.y
	}

	#[inline]
	pub fn contains_xy(&self, x: S, y: S) -> bool {
		self.min.x <= x && x <= self.max.x &&
		self.min.y <= y && y <= self.max.y
	}

	#[inline]
	pub fn contains_rect(&self, r: &Self) -> bool {
		self.contains(r.min) && self.contains(r.max)
	}

	#[inline]
	pub fn inset(self, n: S) -> Self {
		self.inset_xy(n, n)
	}

	#[inline]
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

	#[inline]
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

	#[inline]
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

	#[inline]
	fn is_empty(&self) -> bool {
		self.min.x >= self.max.x || self.min.y >= self.max.y
	}

	#[inline]
	fn min(a: S, b: S) -> S {
		if a < b {
			a
		} else {
			b
		}
	}

	#[inline]
	fn max(a: S, b: S) -> S {
		if a > b {
			a
		} else {
			b
		}
	}

	#[inline]
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

	#[inline]
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

	#[inline]
	pub fn union_point(self, p: Point2<S>) -> Self {
		self.union_xy(p.x, p.y)
	}

	#[inline]
	pub fn union_xy(mut self, x: S, y: S) -> Self {
		self.min.x = Self::min(self.min.x, x);
		self.min.y = Self::min(self.min.y, y);
		self.max.x = Self::max(self.max.x, x);
		self.max.y = Self::max(self.max.y, y);
		self
	}

	#[inline]
	pub fn translate(&self, p: Vector2<S>) -> Self {
		let min = self.min + p;
		let max = self.max + p;
		Self { min, max }
	}

	#[inline]
	pub fn min_translate(&self, p: Vector2<S>) -> Point2<S> {
		self.min + p
	}

	#[inline]
	pub fn min_translate_rect(&self, r: Self) -> Self {
		let from = Vector2::new(self.min.x, self.min.y);
		let min = r.min + from;
		let max = r.max + from;
		Self { min, max }
	}

	#[inline]
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
}
