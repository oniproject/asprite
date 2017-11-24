extern crate cgmath;
#[macro_use] extern crate derivative;

pub use cgmath::prelude::*;
pub use cgmath::{BaseNum, BaseFloat};

pub use cgmath::Transform;

pub use cgmath::Point2;
pub use cgmath::Vector2;

pub use cgmath::Matrix2;
pub use cgmath::Matrix4;

pub mod affine;
pub mod rect;
pub mod d8;
pub mod time;
pub mod stopwatch;

pub use rect::*;
pub use affine::*;
pub use time::*;
pub use stopwatch::*;

use std::ops::Neg;

pub trait BaseNumExt: BaseNum + Neg<Output=Self> {
	#[inline]
	fn abs(self) -> Self {
		if Self::zero() >= self {
			self
		} else {
			-self
		}
	}

	#[inline]
	fn signum(self) -> Self {
		if Self::zero() == self {
			self
		} else if self > Self::one() {
			Self::one()
		} else {
			-Self::one()
		}
	}
}

impl<T: BaseNum + Neg<Output=Self>> BaseNumExt for T {}
