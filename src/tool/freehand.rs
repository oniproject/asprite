use super::*;
use std::cmp;

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

pub struct Rectangle {
	pub start: Point<i16>,
	pub color: u8,
	pub active: bool,
	pub square: bool,
}

impl Rectangle {
	pub fn new() -> Self {
		Self {
			start: Point::new(0, 0),
			color: 0,
			active: false,
			square: false,
		}
	}
}

impl Tool for Rectangle {
	fn run<C: Context>(&mut self, input: Input, ctx: &mut C) {
		match input {
			Input::Move(p) => {
				ctx.sync();
				let mut r = Rect::with_points(p, self.start).normalize();
				if self.square {
					let v = cmp::min(r.max.x, r.max.y);
					r.max.x = v;
					r.max.y = v;
				}
				fill_rect(r, |p| ctx.pixel(p, self.color));
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
			Input::Special(on) => self.line = on,

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
						let r = Rect::with_points(p, last).normalize();
						fill_rect(r, |p| ctx.pixel(p, self.color));
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
			Input::Release(p) => {
				if self.active {
					self.active = false;
					while self.pts.len() > 0 {
						self.flatten_first_point(ctx);
					}
					if self.mode != Mode::Rect {
						ctx.brush(p, self.color);
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