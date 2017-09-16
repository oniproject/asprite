use super::*;

#[derive(Copy, Clone, Debug)]
pub enum Mode {
	Continious,
	PixelPerfect,
	Discontinious,
	Single,
}

#[derive(Clone)]
pub struct Pt {
	pub pt: Point<i16>,
	pub active: bool,
}

pub struct Freehand {
	pub mode: Mode,
	pub last: Point<i16>,
	pub pts: Vec<Pt>,
}

impl Tool for Freehand {
	fn run<C: Context>(&mut self, input: Input, ctx: &mut C) {
		println!("input: {:?}", input);
		match input {
			Input::Move(p) => {
				match self.mode {
					Mode::PixelPerfect => self.update(p, ctx),
					Mode::Continious => {
						let line = Bresenham::new(p, self.last);
						for p in line {
							ctx.brush(p)
						}
					},
					Mode::Discontinious => ctx.brush(p),
					Mode::Single => (),
				}
				self.last = p;
			}

			Input::Press(p) => {
				ctx.brush(p);
				self.pts.push(Pt { pt: p, active: true });
				self.last = p;
			}
			Input::Release(_) => {
				while self.pts.len() > 0 {
					self.flatten_first_point(ctx);
				}
				ctx.commit();
			}
			Input::Cancel => {
				self.pts.clear();
				ctx.rollback();
			}
		}
	}
}

impl Freehand {
	pub fn update<C: Context>(&mut self, m: Point<i16>, ctx: &mut C) {
		if self.point_exists(m.x, m.y) {
			return;
		}

		let line = Bresenham::new(m, self.last.clone());
		for p in line {
			if !self.point_exists(p.x, p.y) {
				self.pts.push(Pt { pt: p, active: true });
			}
		}

		self.cleanup_points();
		while self.pts.len() > 4 {
			self.flatten_first_point(ctx);
		}
	}

	fn flatten_first_point<C: Context>(&mut self, ctx: &mut C) {
		let p = self.pts.remove(0);
		if p.active {
			ctx.brush(p.pt);
		}
		while !self.pts.is_empty() && !self.pts[0].active {
			self.pts.remove(0);
		}
	}

	fn cleanup_points(&mut self) {
		// XXX clone?
		let mut rpts = self.pts.clone();
		rpts.reverse();
		for p in &mut rpts {
			let pt = p.pt;
			let n = self.active_point_exists(pt.x + 0, pt.y - 1);
			let s = self.active_point_exists(pt.x + 0, pt.y + 1);
			let w = self.active_point_exists(pt.x - 1, pt.y + 0);
			let e = self.active_point_exists(pt.x + 1, pt.y + 0);

			let count =
				self.point_exists(pt.x + 0, pt.y - 1) as isize +
				self.point_exists(pt.x + 0, pt.y + 1) as isize +
				self.point_exists(pt.x - 1, pt.y + 0) as isize +
				self.point_exists(pt.x + 1, pt.y + 0) as isize;

			p.active = !(count == 2 && (n && w || n && e || s && w || s && e))
		}
		rpts.reverse();
		self.pts = rpts;
	}

	fn point_exists(&self, x: i16, y: i16) -> bool {
		self.pts.iter().any(|p| p.pt.x == x && p.pt.y == y)
	}

	fn active_point_exists(&self, x: i16, y: i16) -> bool {
		self.pts.iter().any(|p| p.active && p.pt.x == x && p.pt.y == y)
	}
}