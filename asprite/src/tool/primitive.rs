use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveMode {
	Rect,
	Ellipse,
}

pub struct Primitive<N: BaseNum, C: Copy> {
	pub start: Point2<N>,
	pub last: Point2<N>,
	pub color: C,
	pub active: bool,
	pub square: bool,
	pub mode: PrimitiveMode,
	pub fill: bool,
}

impl Primitive<i32, u8> {
	pub fn new() -> Self {
		Self {
			start: Point2::new(0, 0),
			last: Point2::new(0, 0),
			color: 0,
			active: false,
			square: false,
			mode: PrimitiveMode::Rect,
			fill: true,
		}
	}
}

impl<N: BaseNumExt + Step, C: Copy + Clone + Eq> Tool<N, C> for Primitive<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Move(p) => {
				if self.active {
					ctx.sync();
					let p = if self.square {
						let dx = self.start.x - p.x;
						let dy = self.start.y - p.y;
						let d = {
							let dx = dx.abs();
							let dy = dy.abs();
							if dx < dy {
								dx
							} else {
								dy
							}
						};
						Point2::new(
							self.start.x - d * dx.signum(),
							self.start.y - d * dy.signum(),
						)
					} else {
						p
					};
					let mut r = Rect { min: p, max: self.start }
						.normalize();
					r.max += Vector2::new(N::one(), N::one());
					match (self.fill, self.mode) {
						(true, PrimitiveMode::Rect) => ctx.rect_fill(r, self.color),
						(false, PrimitiveMode::Rect) => ctx.rect(r, self.color),
						(false, PrimitiveMode::Ellipse) => ctx.ellipse(r, self.color),
						(true, PrimitiveMode::Ellipse) => ctx.ellipse_fill(r, self.color),
					}
					ctx.update(r.union_point(self.last));
				}
				self.last = p;
			}

			Input::Special(on) => self.square = on,
			Input::Press(p) => {
				self.active = true;
				self.color = ctx.start();
				self.start = p;
				self.last = p;
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