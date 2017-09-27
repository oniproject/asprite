use super::*;

pub struct EyeDropper<N: Signed, C: Copy> {
	pub start: Point<N>,
	pub color: C,
}

impl EyeDropper<i16, u8> {
	pub fn new() -> Self {
		EyeDropper {
			start: Point::new(0, 0),
			color: 0,
		}
	}
}

impl<N: Signed, C: Copy + PartialEq> Tool<N, C> for EyeDropper<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Press(p) => {
				if let Some(color) = ctx.at(p.x, p.y) {
					ctx.change_foreground(color);
				}
			}
			_ => (),
		}
	}
}