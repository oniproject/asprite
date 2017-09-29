use rand;
use std::num::Wrapping;

pub struct Gradient {
	// First color of the gradient.
	pub lower_bound: i32,
	// Last color of the gradient
	pub upper_bound: i32,
	// true if the gradient should use colors in descending order
	pub is_inverted: bool,
	// Number of colors in the range ::lower_bound to ::upper_bound (included)
	pub bounds_range: i32,
	// Maximum value passed to the gradient function. The pixels assigned this value should use last gradient color.
	pub total_range: i32,
	// Amount of randomness to use in gradient (1-256+)
	pub random_factor: i32,
	// Gradient speed of cycling (0-64)
	pub speed: u8,

	// Pointer to a gradient function, depending on the selected method.
	//function: Box<FnMut(i32, i16, i16)>,

	// Pointer to the pixel-drawing function that gradients should use:
	// either ::Pixel (if the gradient must be drawn on menus only)
	// or ::Display_pixel (if the gradient must be drawn on the image)
	pub pixel: Box<FnMut(u16, u16, u8)>,
	// Index in ::T_Page::Gradients of the currently selected gradient.
	pub current: u8,
	// Boolean, true when the color cycling is active.
	pub cycling: u8,
}

impl Gradient {
	pub fn new(pixel: Box<FnMut(u16, u16, u8)>) -> Self {
		Self {
			pixel,

			lower_bound: 0,
			upper_bound: 4,
			is_inverted: false,
			bounds_range: 4,
			total_range: 4,
			random_factor: 1,
			speed: 0,
			current: 0,
			cycling: 0,
		}
	}

	fn px(&mut self, x: i16, y: i16, position: i32) {
		let c = self.pos_to_color(position);
		(self.pixel)(x as u16, y as u16, c);
	}

	fn pos_to_color(&self, mut position: i32) -> u8 {
		if position < 0 {
			position = 0;
		} else if position >= self.bounds_range {
			position = self.bounds_range - 1;
		}
		if self.is_inverted {
			(self.upper_bound - position) as u8
		} else {
			(self.lower_bound + position) as u8
		}
	}

	fn pos_from_idx(&self, i: i32) -> i32 {
		let mut pos = Wrapping(i) * Wrapping(self.bounds_range);
		let rnd = rand::random::<i32>();
		pos += (Wrapping(self.total_range) * Wrapping(rnd % self.random_factor)) >> 6;
		pos -= Wrapping(self.total_range * self.random_factor) >> 7;
		if pos < Wrapping(0) {
			pos = Wrapping(0);
		}
		pos.0
	}

	pub fn basic(&mut self, index: i32, x: i16, y: i16) {
		let mut position = self.pos_from_idx(index);
		position /= self.total_range;
		self.px(x, y, position);
	}

	pub fn dithered(&mut self, index: i32, x: i16, y: i16) {
		let mut position = self.pos_from_idx(index);
		let mut segment = ((position * 4) / self.total_range) % 4;
		position /= self.total_range;

		self.px(x, y, match segment {
			0 if (x + y) & 1 == 0 => position - 1,
			3 if (x + y) & 1 != 0 => position + 1,
			_ => position,
		})
	}

	pub fn extra_dithered(&mut self, index: i32, x: i16, y: i16) {
		let mut position = self.pos_from_idx(index);
		let segment = ((position * 8) / self.total_range) % 8;
		position /= self.total_range;
		self.px(x, y, match segment {
			0     if (x + y) & 1 == 0         => position - 1,
			1 | 2 if x & 1 == 0 && y & 1 == 0 => position - 1,
			5 | 6 if x & 1 == 0 && y & 1 != 0 => position + 1,
			7     if (x + y) & 1 != 0         => position + 1,
			_ => position, 
		})
	}
}
