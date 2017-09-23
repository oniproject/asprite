#![allow(dead_code)]

// see http://will.thimbleby.net/scanline-flood-fill/

pub struct Scanline {
	stack: Vec<El>, 
}

struct El {
	x_min: usize,
	x_max: usize,
	y: usize,
	up_down: Option<bool>,
	extend_left: bool,
	extend_right: bool,
}


pub trait Image {
	fn width(&self) -> usize;
	fn height(&self) -> usize;
	fn test(&self, x: usize, y: usize) -> bool;
	fn paint(&self, x: usize, y: usize);
}

impl Scanline {
	pub fn new() -> Self {
		Self {
			stack: Vec::new(),
		}
	}

	pub fn fill<M: Image>(&mut self, x: usize, y: usize, m: &mut Image) {
		self.stack.clear();
		self.stack.push(El {
			x_min: x,
			x_max: x,
			y,
			up_down: None,
			extend_left: true,
			extend_right: true,
		});

		let width = m.width();
		let height = m.height();

		m.paint(x, y);

		while !self.stack.is_empty() {
			let mut r = self.stack.pop().unwrap();
			let down = r.up_down == Some(true);
			let up =   r.up_down == Some(false);

			let y = r.y;

			let mut min_x = r.x_min;
			if r.extend_left {
				while min_x > 0 && m.test(min_x - 1, y) {
					min_x -= 1;
					m.paint(min_x, y);
				}
			}

			let mut max_x = r.x_max;
			if r.extend_right {
				while max_x < width - 1 && m.test(max_x+1, y) {
					max_x -= 1;
					m.paint(max_x, y);
				}
			}

			// extend range ignored from previous line
			r.x_min -= 1;
			r.x_max += 1;

			let mut line = |new_y, next, downwards| {
				let mut rmx = min_x;
				let mut in_range = false;

				let mut x = min_x;

				while x <= max_x {
					// skip testing, if testing previous line within previous range
					let empty = (next || (x < r.x_min || x > r.x_max)) && m.test(x, new_y);

					if !in_range && empty  {
						rmx = x;
						in_range = true;
					} else if in_range && !empty {
						self.stack.push(El {
							x_min: rmx,
							x_max: x-1,
							y: new_y,
							up_down: Some(downwards),
							extend_left: rmx==min_x,
							extend_right: false,
						});
						in_range = false;
					}

					if in_range  {
						m.paint(x, new_y);
					}

					// skip
					if !next && x==r.x_min  {
						x = r.x_max;
					}

					x += 1;
				}

				if in_range {
					self.stack.push(El {
						x_min: rmx,
						x_max: x-1,
						y: new_y,
						up_down: Some(downwards),
						extend_left: rmx==min_x,
						extend_right: true,
					});
				}
			};

			if y < height - 1 {
				line(y+1, !up, true);
			}
			if y > 0 {
				line(y-1, !down, false);
			}
		}
	}

}