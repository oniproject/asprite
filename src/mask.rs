#![allow(dead_code)]
use common::*;

pub struct Mask {
	pub pix: Vec<bool>,
	pub w: usize,
	pub h: usize,
}

impl Mask {
	pub fn new_square(w: usize, h: usize) -> Self {
		Self {
			w, h,
			pix: vec![true; w*h],
		}
	}

	pub fn draw<F>(&self, pixel: F)
		where F: Fn(i16, i16, bool)
	{
		let mut ptr = self.pix.as_ptr();
		for y in 0..self.h {
			for x in 0..self.w {
				pixel(x as i16, y as i16, unsafe { *ptr });
				ptr = unsafe { ptr.offset(1) };
			}
		}
	}

	pub fn draw_at<F>(&self, at: Point<i16>, pixel: F)
		where F: Fn(i16, i16)
	{
		let ex = at.x + self.w as i16;
		let ey = at.y + self.h as i16;
		let mut ptr = self.pix.as_ptr();
		for y in at.y..ey {
			for x in at.x..ex {
				if unsafe { *ptr } {
					pixel(x, y);
				}
				ptr = unsafe { ptr.offset(1) };
			}
		}
	}
}
