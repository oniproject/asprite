use bre::*;
use math::*;

#[derive(Clone)]
pub struct Pt {
	pub active: bool,
	pub x: i16,
	pub y: i16,
}

pub struct BB {
	pub grid: Vec<bool>,
	pub pts: Vec<Pt>,

	pub zoom: i16,
	pub pos: Point<i16>,
	pub size: Point<i16>,

	pub last: Option<Point<i16>>,

	pub drawing: bool,
}

impl BB {
	pub fn update(&mut self, m: Point<i16>) -> bool {
		if !self.drawing {
			return false;
		}

		if !(m.x >= 0 && m.x < self.size.x && m.y >= 0 && m.y < self.size.y && !self.point_exists(m.x, m.y)) {
			return false;
		}

		match self.last {
			Some(ref last) => {
				let line = Bresenham::new(m, last.clone());
				for p in line {
					if !self.point_exists(p.x, p.y) {
						self.pts.push(Pt {
							x: p.x,
							y: p.y,
							active: true,
						});
					}
				}
			}

			None => {
				self.pts.push(Pt {
					x: m.x,
					y: m.y,
					active: true,
				});
			}
		}

		self.last = Some(m);

		self.cleanup_points();
		while self.pts.len() > 4 {
			self.flatten_first_point();
		}

		true
	}

	pub fn up(&mut self) {
		self.drawing = false;
		self.last = None;
		while self.pts.len() > 0 {
			self.flatten_first_point();
		}
	}

	pub fn down(&mut self) {
		self.drawing = true;
	}

	fn flatten_first_point(&mut self) {
		let pt = self.pts.remove(0);
		if pt.active {
			let idx = pt.x + pt.y * self.size.x;
			self.grid[idx as usize] = true;
		}
		while !self.pts.is_empty() && !self.pts[0].active {
			self.pts.remove(0);
		}
	}

	fn cleanup_points(&mut self) {
		// XXX clone?
		let mut rpts = self.pts.clone();
		rpts.reverse();
		for pt in &mut rpts {
			let n = self.active_point_exists(pt.x + 0, pt.y - 1);
			let s = self.active_point_exists(pt.x + 0, pt.y + 1);
			let w = self.active_point_exists(pt.x - 1, pt.y + 0);
			let e = self.active_point_exists(pt.x + 1, pt.y + 0);

			let count =
				self.point_exists(pt.x + 0, pt.y - 1) as isize +
				self.point_exists(pt.x + 0, pt.y + 1) as isize +
				self.point_exists(pt.x - 1, pt.y + 0) as isize +
				self.point_exists(pt.x + 1, pt.y + 0) as isize;

			pt.active = !(count == 2 && (n && w || n && e || s && w || s && e))
		}
		rpts.reverse();
		self.pts = rpts;
	}

	fn point_exists(&self, x: i16, y: i16) -> bool {
		self.pts.iter().any(|pt| pt.x == x && pt.y == y)
	}

	fn active_point_exists(&self, x: i16, y: i16) -> bool {
		self.pts.iter().any(|pt| pt.active && pt.x == x && pt.y == y)
	}
}

