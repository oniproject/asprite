use math::*;

pub trait MouseEvent {
	fn was_pressed(&self) -> bool;
	fn was_released(&self) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct Mouse {
	pub cursor: Point2<f32>,
	pub pressed: [bool; 3],
	pub released: [bool; 3],
}

impl MouseEvent for Mouse {
	#[inline]
	fn was_pressed(&self) -> bool {
		self.pressed[0]
	}
	#[inline]
	fn was_released(&self) -> bool {
		self.released[0]
	}
}

impl Mouse {
	#[inline]
	pub fn new() -> Self {
		Self {
			cursor: Point2::new(0.0, 0.0),
			pressed: [false; 3],
			released: [false; 3],
		}
	}

	#[inline]
	pub fn cursor(&self) -> Point2<f32> {
		self.cursor
	}

	#[inline]
	pub fn cursor_in_rect(&self, r: &Rect<f32>) -> Option<Point2<f32>> {
		if r.contains(self.cursor) {
			Some(self.cursor)
		} else {
			None
		}
	}

	#[inline]
	pub fn check_cursor(&self, r: &Rect<f32>) -> bool {
		r.contains(self.cursor)
	}

	#[inline]
	pub fn cleanup(&mut self) {
		self.pressed = [false; 3];
		self.released = [false; 3];
	}
}
