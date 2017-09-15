#![allow(dead_code)]
use common::*;

pub struct Mask {
	pix: Vec<bool>,
	w: usize,
	h: usize,
}

impl Mask {
	pub fn new_square(w: usize, h: usize) -> Self {
		Self {
			w, h,
			pix: vec![true; w*h],
		}
	}

	pub fn draw<F>(&self, at: Point<i16>, pixel: F)
		where F: Fn(i16, i16)
	{
		let ex = at.x + self.w as i16;
		let ey = at.y + self.h as i16;
		let mut ptr = self.pix.as_ptr();
		for y in at.y..ey {
			for x in at.x..ex {
				unsafe {
					if *ptr {
						pixel(x, y);
					}
					ptr = ptr.offset(1);
				}
			}
		}
	}
}
