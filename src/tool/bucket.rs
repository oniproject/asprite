use super::*;

pub struct Bucket<N: BaseNum, C: Copy> {
	pub start: Point2<N>,
	pub color: C,
}

impl Bucket<i32, u8> {
	pub fn new() -> Self {
		Self {
			start: Point2::new(0, 0),
			color: 0,
		}
	}
}

impl<N: BaseNumExt + Step, C: Copy + Clone + Eq> Tool<N, C> for Bucket<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Press(p) => {
				let color = ctx.start();
				ctx.scanline_fill(p, color);
				ctx.commit();
				let r = ctx.bounds();
				ctx.update(r);
			}
			_ => (),
		}
	}
}