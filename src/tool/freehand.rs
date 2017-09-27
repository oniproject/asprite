use super::*;
use std::cmp;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
	Continious,
	PixelPerfect,
	Line,
	Rect,
}


pub struct Rectangle<N: Signed, C: Copy> {
	pub start: Point<N>,
	pub color: C,
	pub active: bool,
	pub square: bool,
}

impl Rectangle<i16, u8> {
	pub fn new() -> Self {
		Self {
			start: Point::new(0, 0),
			color: 0,
			active: false,
			square: false,
		}
	}
}

impl<N: Signed, C: Copy> Tool<N, C> for Rectangle<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Move(p) => {
				ctx.sync();
				let mut r = Rect::with_points(p, self.start).normalize();
				if self.square {
					let v = cmp::min(r.max.x, r.max.y);
					r.max.x = v;
					r.max.y = v;
				}
				ctx.fill_rect(r, self.color);
			}

			Input::Special(on) => self.square = on,
			Input::Press(p) => {
				assert!(!self.active);
				self.color = ctx.start();
				self.start = p;
			}
			Input::Release(_) => ctx.commit(),
			Input::Cancel => ctx.rollback(),
		}
	}
}

pub struct Freehand<N: Signed, C: Copy> {
	pub perfect: bool,
	pub line: bool,

	pub last: Point<N>,
	pub pts: Vec<(Point<N>, bool)>,
	pub color: C,
	pub active: bool,
}

impl<N: Signed, C: Copy> Tool<N, C> for Freehand<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Move(p) => {
				if self.line {
					ctx.sync();
					ctx.draw_line(p, self.last, self.color);
				} else if self.active {
					let last = self.last;
					if self.perfect {
						self.update(p, last, ctx);
					} else {
						ctx.draw_line(p, last, self.color);
					}
					self.last = p;
				}
			}
			Input::Special(on) => {
				self.line = on && !self.active;
				if !self.active {
					ctx.sync();
				}
			}
			Input::Press(p) => {
				assert!(!self.active);
				if self.line {
				} else {
					self.active = true;
					self.color = ctx.start();
					ctx.brush(p, self.color);
					self.pts.push((p, true));
					self.last = p;
				}
			}
			Input::Release(p) => {
				self.last = p;
				if self.active {
					self.active = false;
					while self.pts.len() > 0 {
						self.flatten_first_point(ctx);
					}
				}
				ctx.brush(p, self.color);
				ctx.commit();
			}
			Input::Cancel => {
				self.active = false;
				self.pts.clear();
				ctx.rollback();
			}
		}
	}
}

impl<N: Signed, C: Copy> Freehand<N, C> {
	pub fn update<Ctx: Context<N, C>>(&mut self, m: Point<N>, last: Point<N>, ctx: &mut Ctx) {
		if self.point_exists(m.x, m.y) {
			return;
		}

		draw_line(last, m, |p: Point<N>| {
			if !self.point_exists(p.x, p.y) {
				self.pts.push((p, true));
			}
		});

		self.cleanup_points();
		while self.pts.len() > 4 {
			self.flatten_first_point(ctx);
		}
	}

	fn flatten_first_point<Ctx: Context<N, C>>(&mut self, ctx: &mut Ctx) {
		let p = self.pts.remove(0);
		if p.1 {
			ctx.brush(p.0, self.color);
		}
		while !self.pts.is_empty() && !self.pts[0].1 {
			self.pts.remove(0);
		}
	}

	fn cleanup_points(&mut self) {
		// XXX clone?
		let mut pts = self.pts.clone();
		pts.reverse();
		let o = N::one();
		let z = N::zero();
		for p in &mut pts {
			let pt = p.0;
			let n = self.active_point_exists(pt.x + z, pt.y - o);
			let s = self.active_point_exists(pt.x + z, pt.y + o);
			let w = self.active_point_exists(pt.x - o, pt.y + z);
			let e = self.active_point_exists(pt.x + o, pt.y + z);

			let count =
				self.point_exists(pt.x + z, pt.y - o) as isize +
				self.point_exists(pt.x + z, pt.y + o) as isize +
				self.point_exists(pt.x - o, pt.y + z) as isize +
				self.point_exists(pt.x + o, pt.y + z) as isize;

			p.1 = !(count == 2 && (n && w || n && e || s && w || s && e))
		}
		pts.reverse();
		self.pts = pts;
	}

	fn point_exists(&self, x: N, y: N) -> bool {
		self.pts.iter().any(|p| p.0.x == x && p.0.y == y)
	}

	fn active_point_exists(&self, x: N, y: N) -> bool {
		self.pts.iter().any(|p| p.1 && p.0.x == x && p.0.y == y)
	}
}