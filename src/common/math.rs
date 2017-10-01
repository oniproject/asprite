use num_traits;

use std::iter::Step;
use std::fmt::Debug;
use std::mem::swap;

use na::{Point2, Vector2};

pub type Point<N> = Point2<N>;
pub type Vector<N> = Vector2<N>;

pub trait Signed:
	num_traits::sign::Signed +
	num_traits::NumAssign +
	num_traits::int::PrimInt +
	Ord +
	PartialOrd<Self> +
	Copy +
	Step +
	Debug + 'static
{}

impl Signed for i16 {}
impl Signed for i32 {}
impl Signed for i64 {}


#[derive(Copy, Clone)]
pub struct Rect<T: Signed> {
	pub min: Point<T>,
	pub max: Point<T>,
}

impl<T: Signed> Rect<T> {
	pub fn new() -> Self {
		Self {
			min: Point::new(T::zero(), T::zero()),
			max: Point::new(T::zero(), T::zero()),
		}
	}

	pub fn pos(self, x: T, y: T) -> Self {
		let p = Vector::new(x, y);
		Self {
			min: Point::from_coordinates(self.min.coords + p),
			max: Point::from_coordinates(self.max.coords + p),
		}
	}

	pub fn size(self, w: T, h: T) -> Self {
		let p = Vector::new(w, h);
		Self {
			min: self.min,
			max: Point::from_coordinates(self.min.coords + p),
		}
	}

	pub fn set_w(self, w: T) -> Self {
		self.size(w, self.h())
	}
	pub fn set_h(self, h: T) -> Self {
		self.size(self.w(), h)
	}

	pub fn with_points(min: Point<T>, max: Point<T>) -> Self {
		Self { min, max }
	}
	pub fn with_coords(x1: T, y1: T, x2: T, y2: T) -> Self {
		Self {
			min: Point::new(x1, y1),
			max: Point::new(x2, y2),
		}
	}
	pub fn with_size(x: T, y: T, w: T, h: T) -> Self {
		Self {
			min: Point::new(x, y),
			max: Point::new(x + w, y + h),
		}
	}

	pub fn contains(&self, p: Point<T>) -> bool {
		self.min.x <= p.x && p.x <= self.max.x &&
		self.min.y <= p.y && p.y <= self.max.y
	}

	pub fn contains_rect(&self, r: Self) -> bool {
		self.contains(r.min) && self.contains(r.max)
	}

	pub fn w(&self) -> T { self.max.x - self.min.x }
	pub fn h(&self) -> T { self.max.y - self.min.y }


	pub fn min_translate(&self, p: Point<T>) -> Point<T> {
		Point::from_coordinates(self.min.coords + p.coords)
	}

	pub fn min_translate_rect(&self, r: Self) -> Self {
		let min = Point::from_coordinates(self.min.coords + r.min.coords);
		let max = Point::from_coordinates(self.min.coords + r.max.coords);
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
}
