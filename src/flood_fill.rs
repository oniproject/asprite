use common::*;

// see http://will.thimbleby.net/scanline-flood-fill/

pub trait Scanline<C: PartialEq + Copy> {
	fn width(&self) -> usize;
	fn height(&self) -> usize;
	fn at(&self, x: i16, y: i16) -> C;
	fn paint(&mut self, x: i16, y: i16, color: C);

	fn fill(&mut self, p: Point<i16>, color: C) {
		let x = p.x;
		let y = p.y;

		// x_min, x_max, y, down[true] / up[false], extend_left, extend_right
		let mut ranges = vec![(x, x, y, None, true, true)];
		let test = self.at(x, y);
		let width = self.width() as i16;
		let height = self.height() as i16;
		self.paint(x, y, color);

		while let Some((mut r0, mut r1, y, r3, extend_left, extend_right)) = ranges.pop() {
			let down = r3 == Some(true);
			let up =   r3 == Some(false);

			// extend left
			let mut min_x = r0;
			if extend_left {
				while min_x > 0 && self.at(min_x-1, y) == test {
					min_x -= 1;
					self.paint(min_x, y, color);
				}
			}

			// extend right
			let mut max_x = r1;
			if extend_right {
				while max_x < width-1 && self.at(max_x+1, y) == test {
					max_x += 1;
					self.paint(max_x, y, color);
				}
			}

			// extend range ignored from previous line
			r0 -= 1;
			r1 += 1;

			let mut line = |y, is_next, downwards: bool| {
				let mut rmin = min_x;
				let mut in_range = false;

				let mut x = min_x;

				while x <= max_x {
					// skip testing, if testing previous line within previous range
					let empty = (is_next || x < r0 || x > r1) && self.at(x, y) == test;

					in_range = if !in_range && empty {
						rmin = x;
						true
					} else if in_range && !empty {
						ranges.push((rmin, x-1, y, Some(downwards), rmin == min_x, false));
						false
					} else {
						in_range
					};

					if in_range {
						self.paint(x, y, color);
					}

					// skip
					if !is_next && x == r0 {
						x = r1;
					}

					x += 1;
				}

				if in_range {
					ranges.push((rmin, x-1, y, Some(downwards), rmin == min_x, true));
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