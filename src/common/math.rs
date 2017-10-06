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


#[derive(Copy, Clone, Debug)]
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

	pub fn xy(self, x: T, y: T) -> Self {
		let p = Vector::new(x, y);
		Self {
			min: Point::from_coordinates(self.min.coords + p),
			max: Point::from_coordinates(self.max.coords + p),
		}
	}

	pub fn x(self, x: T) -> Self {
		let p = Vector::new(x, T::zero());
		Self {
			min: Point::from_coordinates(self.min.coords + p),
			max: Point::from_coordinates(self.max.coords + p),
		}
	}

	pub fn y(self, y: T) -> Self {
		let p = Vector::new(T::zero(), y);
		Self {
			min: Point::from_coordinates(self.min.coords + p),
			max: Point::from_coordinates(self.max.coords + p),
		}
	}

	pub fn w(self, w: T) -> Self {
		let p = Vector::new(w, self.dy());
		Self {
			min: self.min,
			max: Point::from_coordinates(self.min.coords + p),
		}
	}

	pub fn h(self, h: T) -> Self {
		let p = Vector::new(self.dx(), h);
		Self {
			min: self.min,
			max: Point::from_coordinates(self.min.coords + p),
		}
	}

	pub fn wh(self, w: T, h: T) -> Self {
		let p = Vector::new(w, h);
		Self {
			min: self.min,
			max: Point::from_coordinates(self.min.coords + p),
		}
	}

	pub fn set_w(self, w: T) -> Self {
		self.wh(w, self.dy())
	}
	pub fn set_h(self, h: T) -> Self {
		self.wh(self.dx(), h)
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

	pub fn dx(&self) -> T { self.max.x - self.min.x }
	pub fn dy(&self) -> T { self.max.y - self.min.y }

	pub fn inset(self, n: T) -> Self {
		self.inset_xy(n, n)
	}

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

	fn empty(&self) -> bool {
		self.min.x >= self.max.x || self.min.y >= self.max.y
	}

	pub fn intersect(mut self, s: Self) -> Option<Self> {
		self.min.x = self.min.x.max(s.min.x);
		self.min.y = self.min.y.max(s.min.y);
		self.max.x = self.max.x.min(s.max.x);
		self.max.y = self.max.y.min(s.max.y);
		if self.empty() {
			None
		} else {
			Some(self)
		}
	}

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
