use super::*;

pub struct Bucket<N: Signed, C: Copy> {
	pub start: Point<N>,
	pub color: C,
}

impl Bucket<i16, u8> {
	pub fn new() -> Self {
		Self {
			start: Point::new(0, 0),
			color: 0,
		}
	}
}

impl<N: Signed, C: Copy + PartialEq> Tool<N, C> for Bucket<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Press(p) => {
				let color = ctx.start();
				ctx.fill(p, color);
				ctx.commit();
			}
			_ => (),
		}
	}
}