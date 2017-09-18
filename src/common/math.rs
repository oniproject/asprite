use num_traits;

use std::iter::Step;
use std::fmt::Debug;

use na::Point2;

pub type Point<N> = Point2<N>;

pub trait Signed:
	num_traits::sign::Signed +
	num_traits::NumAssign +
	PartialOrd<Self> +
	Copy +
	Step +
	Debug + 'static
{}

impl Signed for i8 {}
impl Signed for i16 {}
impl Signed for i32 {}
impl Signed for i64 {}


#[derive(Copy, Clone)]
pub struct Rect<T: Signed> {
	pub min: Point<T>,
	pub max: Point<T>,
}

impl<T: Signed> Rect<T> {
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

	pub fn w(&self) -> T { self.max.x - self.min.x }
	pub fn h(&self) -> T { self.max.y - self.min.y }

	pub fn set_w(&mut self, w: T) {
		self.max.x = self.min.x + w;
	}
	pub fn set_h(&mut self, h: T) {
		self.max.y = self.min.y + h;
	}
}
