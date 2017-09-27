use super::*;

// see http://will.thimbleby.net/scanline-flood-fill/

pub trait Image<N: Signed, C: Copy + PartialEq> {
	fn paint_pixel(&mut self, p: Point<N>, color: C);
	fn paint_brush(&mut self, p: Point<N>, color: C);
	fn at(&self, x: N, y: N) -> Option<C>;

	fn width(&self) -> N;
	fn height(&self) -> N;

	fn fill_rect(&mut self, r: Rect<N>, color: C) {
		fill_rect(r, |p| self.paint_pixel(p, color));
	}

	fn draw_line(&mut self, start: Point<N>, end: Point<N>, color: C) {
		draw_line(start, end, |p| self.paint_brush(p, color));
	}

	fn ellipse(&mut self, r: Rect<N>, color: C) {
		let (mut x0, mut y0, mut x1, mut y1) = (
			r.min.x.to_i64().unwrap(),
			r.min.y.to_i64().unwrap(),
			r.max.x.to_i64().unwrap(),
			r.max.y.to_i64().unwrap(),
		); 

		let mut a = (x1-x0).abs();
		let b = (y1-y0).abs();
		// values of diameter
		let mut b1 = b & 1;

		// error increment
		let mut dx = 4*(1-a)*b*b;
		let mut dy = 4*(b1+1)*a*a;
		let mut err = dx+dy+b1*a*a;
		let mut e2; // error of 1.step

		// if called with swapped points
		if x0 > x1 {
			x0 = x1;
			x1 += a;
		}
		// .. exchange them
		if y0 > y1 {
			y0 = y1;
		}
		// starting pixel 
		y0 += (b+1)/2;
		y1 = y0-b1;
		a *= 8*a;
		b1 = 8*b*b;

		while {
			let p = Point::new(N::from(x1).unwrap(), N::from(y0).unwrap());
			self.paint_pixel(p, color); //   I. Quadrant
			let p = Point::new(N::from(x0).unwrap(), N::from(y0).unwrap());
			self.paint_pixel(p, color); //  II. Quadrant
			let p = Point::new(N::from(x0).unwrap(), N::from(y1).unwrap());
			self.paint_pixel(p, color); // III. Quadrant
			let p = Point::new(N::from(x1).unwrap(), N::from(y1).unwrap());
			self.paint_pixel(p, color); //  IV. Quadrant
			e2 = 2*err;
			// y step 
			if e2 <= dy {
				y0 += 1;
				y1 -= 1;
				dy += a;
				err += dy;
			}
			// x step
			if e2 >= dx || 2*err > dy {
				x0 += 1;
				x1 -= 1;
				dx += b1;
				err += dx;
			}

			x0 <= x1
		} {}
		
		// too early stop of flat ellipses a=1
		while y0-y1 < b {
			// -> finish tip of ellipse 
			let p = Point::new(N::from(x0-1).unwrap(), N::from(y0).unwrap());
			self.paint_pixel(p, color);
			let p = Point::new(N::from(x1+1).unwrap(), N::from(y0).unwrap());
			self.paint_pixel(p, color);
			y0 += 1;

			let p = Point::new(N::from(x0-1).unwrap(), N::from(y1).unwrap());
			self.paint_pixel(p, color);
			let p = Point::new(N::from(x1+1).unwrap(), N::from(y1).unwrap());
			self.paint_pixel(p, color);
			y1 -= 1;
		}
	}

	fn scanline_fill(&mut self, p: Point<N>, color: C) {
		let x = p.x;
		let y = p.y;

		let one = N::one();
		let zero = N::zero();

		// x_min, x_max, y, down[true] / up[false], extend_left, extend_right
		let mut ranges = vec![(x, x, y, None, true, true)];
		let width = self.width();
		let height = self.height();

		if x < zero || x >= width - one || y < zero || y >= height {
			return;
		}

		let test = self.at(x, y);
		if test.is_none() {
			return;
		}

		self.paint_pixel(p, color);

		while let Some((mut r0, mut r1, y, r3, extend_left, extend_right)) = ranges.pop() {
			let down = r3 == Some(true);
			let up =   r3 == Some(false);

			// extend left
			let mut min_x = r0;
			if extend_left {
				while min_x > zero && self.at(min_x-one, y) == test {
					min_x -= one;
					self.paint_pixel(Point::new(min_x, y), color);
				}
			}

			// extend right
			let mut max_x = r1;
			if extend_right {
				while max_x < width-one && self.at(max_x+one, y) == test {
					max_x += one;
					self.paint_pixel(Point::new(max_x, y), color);
				}
			}

			// extend range ignored from previous line
			r0 -= one;
			r1 += one;

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
						ranges.push((rmin, x-one, y, Some(downwards), rmin == min_x, false));
						false
					} else {
						in_range
					};

					if in_range {
						self.paint_pixel(Point::new(x, y), color);
					}

					// skip
					if !is_next && x == r0 {
						x = r1;
					}

					x += one;
				}

				if in_range {
					ranges.push((rmin, x-one, y, Some(downwards), rmin == min_x, true));
				}
			};

			if y < height - one {
				line(y+one, !up, true);
			}
			if y > zero {
				line(y-one, !down, false);
			}
		}
	}
}