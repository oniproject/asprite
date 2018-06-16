use super::*;

pub struct EyeDropper<N: BaseNum, C: Copy> {
	pub start: Point2<N>,
	pub color: C,
}

impl EyeDropper<i32, u8> {
	pub fn new() -> Self {
		EyeDropper {
			start: Point2::new(0, 0),
			color: 0,
		}
	}
}

impl<N: BaseNumExt + Step, C: Copy + Clone + Eq> Tool<N, C> for EyeDropper<N, C> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
		match input {
			Input::Press(p) => {
				if let Some(color) = ctx.at(p.x, p.y) {
					ctx.change_color(color);
				}
			}
			_ => (),
		}
	}
}