use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
	Continious,
	PixelPerfect,
	Line,
	Rect,
}

#[derive(Clone)]
pub struct Pt {
	pub pt: Point<i16>,
	pub active: bool,
}

pub struct Freehand {
	pub mode: Mode,
	pub line: bool,

	pub last: Point<i16>,
	pub pts: Vec<Pt>,
	pub color: u8,
	pub active: bool,
}

impl Tool for Freehand {
	fn run<C: Context>(&mut self, input: Input, ctx: &mut C) {
		match input {
			Input::Down(Key::Shift) => self.line = true,
			Input::Up(Key::Shift) => self.line = false,

			Input::Move(p) => {
				let last = self.last;
				self.last = p;
				match self.mode {
					Mode::PixelPerfect => self.update(p, last, ctx),
					Mode::Continious => {
						draw_line(p, last, |p| ctx.brush(p, self.color))
					}
					Mode::Line => {
						self.last = last;
						ctx.sync();
						draw_line(p, last, |p| ctx.brush(p, self.color))
					}
					Mode::Rect => {
						self.last = last;
						ctx.sync();
						let (mut sx, mut sy) = (p.x, p.y);
						let (mut ex, mut ey) = (last.x, last.y);
						if sx > ex {
							::std::mem::swap(&mut sx, &mut ex);
						}
						if sy > ey {
							::std::mem::swap(&mut sy, &mut ey);
						}
						for y in sy..ey {
							for x in sx..ex {
								ctx.pixel(Point::new(x, y), self.color);
							}
						}
					}
				}
			}

			Input::Press(p) => {
				assert!(!self.active);
				self.active = true;
				self.color = ctx.start();
				if self.mode != Mode::Rect {
					ctx.brush(p, self.color);
					self.pts.push(Pt { pt: p, active: true });
				}
				self.last = p;
			}
			Input::Release(_) => {
				if self.active {
					self.active = false;
					while self.pts.len() > 0 {
						self.flatten_first_point(ctx);
					}
					ctx.commit();
				}
			}
			Input::Cancel => {
				self.active = false;
				self.pts.clear();
				ctx.rollback();
			}
		}
	}
}

impl Freehand {
	pub fn update<C: Context>(&mut self, m: Point<i16>, last: Point<i16>, ctx: &mut C) {
		if self.point_exists(m.x, m.y) {
			return;
		}

		draw_line(last, m, |p|{
			if !self.point_exists(p.x, p.y) {
				self.pts.push(Pt { pt: p, active: true });
			}
		});

		self.cleanup_points();
		while self.pts.len() > 4 {
			self.flatten_first_point(ctx);
		}
	}

	fn flatten_first_point<C: Context>(&mut self, ctx: &mut C) {
		let p = self.pts.remove(0);
		if p.active {
			ctx.brush(p.pt, self.color);
		}
		while !self.pts.is_empty() && !self.pts[0].active {
			self.pts.remove(0);
		}
	}

	fn cleanup_points(&mut self) {
		// XXX clone?
		let mut pts = self.pts.clone();
		pts.reverse();
		for p in &mut pts {
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
		pts.reverse();
		self.pts = pts;
	}

	fn point_exists(&self, x: i16, y: i16) -> bool {
		self.pts.iter().any(|p| p.pt.x == x && p.pt.y == y)
	}

	fn active_point_exists(&self, x: i16, y: i16) -> bool {
		self.pts.iter().any(|p| p.active && p.pt.x == x && p.pt.y == y)
	}
}