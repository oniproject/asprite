use super::*;
use std::cmp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveMode {
	Rect,
	Ellipse,
}

pub struct Primitive<N: Signed, C: Copy> {
	pub start: Point<N>,
	pub color: C,
	pub active: bool,
	pub square: bool,
	pub mode: PrimitiveMode,
	pub fill: bool,
}

impl Primitive<i16, u8> {
	pub fn new() -> Self {
		Primitive {
			start: Point::new(0, 0),
			color: 0,
			active: false,
			square: false,
			mode: PrimitiveMode::Rect,
			fill: true,
		}
	}
}

impl<N: Signed, C: Copy + PartialEq> Tool<N, C> for Primitive<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Move(p) => {
				if self.active {
					ctx.sync();
					let p = if self.square {
						let dx = self.start.x - p.x;
						let dy = self.start.y - p.y;
						let d = cmp::min(dx.abs(), dy.abs());
						Point::new(
							self.start.x - d * dx.signum(),
							self.start.y - d * dy.signum(),
						)
					} else {
						p
					};
					let r = Rect::with_points(p, self.start).normalize();
					match (self.fill, self.mode) {
						(true, PrimitiveMode::Rect) => ctx.fill_rect(r, self.color),
						(false, PrimitiveMode::Rect) => ctx.draw_rect(r, self.color),
						(false, PrimitiveMode::Ellipse) => ctx.draw_ellipse(r, self.color),
						(true, PrimitiveMode::Ellipse) => ctx.fill_ellipse(r, self.color),
					}
				}
			}

			Input::Special(on) => self.square = on,
			Input::Press(p) => {
				assert!(!self.active);
				self.active = true;
				self.color = ctx.start();
				self.start = p;
			}
			Input::Release(_) => {
				self.active = false;
				ctx.commit();
			}
			Input::Cancel => {
				self.active = false;
				ctx.rollback();
			}
		}
	}
}