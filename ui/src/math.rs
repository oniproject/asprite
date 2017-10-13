use num_traits;

use std::iter::Step;
use std::fmt::Debug;
use std::mem::swap;

use na::{Point2, Vector2};

pub type Point<N> = Point2<N>;
pub type Vector<N> = Vector2<N>;

pub trait Num:
	num_traits::sign::Signed +
	num_traits::NumCast +
	num_traits::NumAssign +
	PartialOrd<Self> +
	Copy +
	Debug + 'static
{
	#[inline(always)]
	fn min(self, b: Self) -> Self {
		if self < b { self } else { b }
	}

	#[inline(always)]
	fn max(self, b: Self) -> Self {
		if self > b { self } else { b }
	}
}

pub trait Float:
	Num + num_traits::float::Float
{
	#[inline(always)]
	fn lerp(v0: Self, v1: Self, t: Self) -> Self {
		(Self::one() - t) * v0 + t * v1
	}
}

pub trait SignedInt:
	Num + num_traits::int::PrimInt + Ord + Step
{}

impl Num for f32 {}
impl Num for f64 {}
impl Num for i16 {}
impl Num for i32 {}
impl Num for i64 {}

impl Float for f32 {}
impl Float for f64 {}

impl SignedInt for i16 {}
impl SignedInt for i32 {}
impl SignedInt for i64 {}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Rect<T: Num> {
	pub min: Point<T>,
	pub max: Point<T>,
}

impl<T: Num> Default for Rect<T> {
	#[inline(always)]
	fn default() -> Self {
		Self {
			min: Point::new(T::zero(), T::zero()),
			max: Point::new(T::zero(), T::zero()),
		}
	}
}

impl<T: Num> Rect<T> {
	#[inline(always)]
	pub fn new() -> Self {
		Self {
			min: Point::new(T::zero(), T::zero()),
			max: Point::new(T::zero(), T::zero()),
		}
	}

	#[inline(always)]
	pub fn xy(self, x: T, y: T) -> Self {
		let p = Vector::new(x, y);
		Self {
			min: Point::from_coordinates(self.min.coords + p),
			max: Point::from_coordinates(self.max.coords + p),
		}
	}

	#[inline(always)]
	pub fn x(self, x: T) -> Self {
		let p = Vector::new(x, T::zero());
		Self {
			min: Point::from_coordinates(self.min.coords + p),
			max: Point::from_coordinates(self.max.coords + p),
		}
	}

	#[inline(always)]
	pub fn y(self, y: T) -> Self {
		let p = Vector::new(T::zero(), y);
		Self {
			min: Point::from_coordinates(self.min.coords + p),
			max: Point::from_coordinates(self.max.coords + p),
		}
	}

	#[inline(always)]
	pub fn w(self, w: T) -> Self {
		let p = Vector::new(w, self.dy());
		Self {
			min: self.min,
			max: Point::from_coordinates(self.min.coords + p),
		}
	}

	#[inline(always)]
	pub fn h(self, h: T) -> Self {
		let p = Vector::new(self.dx(), h);
		Self {
			min: self.min,
			max: Point::from_coordinates(self.min.coords + p),
		}
	}

	#[inline(always)]
	pub fn wh(self, w: T, h: T) -> Self {
		let p = Vector::new(w, h);
		Self {
			min: self.min,
			max: Point::from_coordinates(self.min.coords + p),
		}
	}

	#[inline(always)]
	pub fn with_coords(x1: T, y1: T, x2: T, y2: T) -> Self {
		Self {
			min: Point::new(x1, y1),
			max: Point::new(x2, y2),
		}
	}
	#[inline(always)]
	pub fn with_size(x: T, y: T, w: T, h: T) -> Self {
		Self {
			min: Point::new(x, y),
			max: Point::new(x + w, y + h),
		}
	}

	#[inline(always)]
	pub fn contains(&self, p: Point<T>) -> bool {
		self.min.x <= p.x && p.x <= self.max.x &&
		self.min.y <= p.y && p.y <= self.max.y
	}

	#[inline(always)]
	pub fn contains_xy(&self, x: T, y: T) -> bool {
		self.min.x <= x && x <= self.max.x &&
		self.min.y <= y && y <= self.max.y
	}

	#[inline(always)]
	pub fn contains_rect(&self, r: Self) -> bool {
		self.contains(r.min) && self.contains(r.max)
	}

	#[inline(always)]
	pub fn dx(&self) -> T { self.max.x - self.min.x }
	#[inline(always)]
	pub fn dy(&self) -> T { self.max.y - self.min.y }

	#[inline(always)]
	pub fn inset(self, n: T) -> Self {
		self.inset_xy(n, n)
	}

	#[inline(always)]
	pub fn inset_xy(self, x: T, y: T) -> Self {
		let Self { mut min, mut max } = self;
		let two = T::one() + T::one();
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

	#[inline(always)]
	pub fn inset_x(self, n: T) -> Self {
		let Self { mut min, mut max } = self;
		let two = T::one() + T::one();
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

	#[inline(always)]
	pub fn inset_y(self, n: T) -> Self {
		let Self { mut min, mut max } = self;
		let two = T::one() + T::one();
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

	#[inline(always)]
	fn empty(&self) -> bool {
		self.min.x >= self.max.x || self.min.y >= self.max.y
	}

	#[inline(always)]
	fn min(a: T, b: T) -> T {
		if a < b {
			a
		} else {
			b
		}
	}

	#[inline(always)]
	fn max(a: T, b: T) -> T {
		if a > b {
			a
		} else {
			b
		}
	}

	#[inline(always)]
	pub fn intersect(mut self, s: Self) -> Option<Self> {
		self.min.x = Self::max(self.min.x, s.min.x);
		self.min.y = Self::max(self.min.y, s.min.y);
		self.max.x = Self::min(self.max.x, s.max.x);
		self.max.y = Self::min(self.max.y, s.max.y);
		if self.empty() {
			None
		} else {
			Some(self)
		}
	}

	#[inline(always)]
	pub fn union(mut self, s: Self) -> Option<Self> {
		if self.empty() || s.empty() {
			None
		} else {
			self.min.x = Self::min(self.min.x, s.min.x);
			self.min.y = Self::min(self.min.y, s.min.y);
			self.max.x = Self::max(self.max.x, s.max.x);
			self.max.y = Self::max(self.max.y, s.max.y);
			Some(self)
		}
	}

	#[inline(always)]
	pub fn union_point(mut self, p: Point<T>) -> Self {
		self.min.x = Self::min(self.min.x, p.x);
		self.min.y = Self::min(self.min.y, p.y);
		self.max.x = Self::max(self.max.x, p.x);
		self.max.y = Self::max(self.max.y, p.y);
		self
	}

	#[inline(always)]
	pub fn union_xy(mut self, x: T, y: T) -> Self {
		self.min.x = Self::min(self.min.x, x);
		self.min.y = Self::min(self.min.y, y);
		self.max.x = Self::max(self.max.x, x);
		self.max.y = Self::max(self.max.y, y);
		self
	}

	#[inline(always)]
	pub fn translate(&self, p: Point<T>) -> Self {
		let min = Point::from_coordinates(self.min.coords + p.coords);
		let max = Point::from_coordinates(self.max.coords + p.coords);
		Self { min, max }
	}

	#[inline(always)]
	pub fn min_translate(&self, p: Point<T>) -> Point<T> {
		Point::from_coordinates(self.min.coords + p.coords)
	}

	#[inline(always)]
	pub fn min_translate_rect(&self, r: Self) -> Self {
		let min = Point::from_coordinates(self.min.coords + r.min.coords);
		let max = Point::from_coordinates(self.min.coords + r.max.coords);
		Self { min, max }
	}

	#[inline(always)]
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

	#[inline(always)]
	pub fn align<F: Float>(&self, x: F, y: F, size: Point<T>) -> Point<T> {
		let (tw, th) = (size.x, size.y);
		let (rw, rh) = (self.dx(), self.dy());

		let dw = F::from(rw - tw).unwrap();
		let dh = F::from(rh - th).unwrap();

		let x = self.min.x + T::from(dw * x).unwrap();
		let y = self.min.y + T::from(dh * y).unwrap();

		Point::new(x, y)
	}
}

impl<F: Float> Rect<F> {
	#[inline(always)]
	pub fn transform(&self, anchor: Self, offset: Self) -> Self {
		let min_x = F::lerp(self.min.x, self.max.x, anchor.min.x) + offset.min.x;
		let min_y = F::lerp(self.min.y, self.max.y, anchor.min.y) + offset.min.y;
		let max_x = F::lerp(self.min.x, self.max.x, anchor.max.x) + offset.max.x;
		let max_y = F::lerp(self.min.y, self.max.y, anchor.max.y) + offset.max.y;
		let min = Point::new(min_x, min_y);
		let max = Point::new(max_x, max_y);
		Rect { min, max }
	}
}