use super::*;

pub struct Progress<BG, F> {
	pub background: BG,
	pub fill: F,
}

impl<D, BG, F> Component<D> for Progress<BG, F>
	where
		D: ?Sized + Graphics,
		BG: FrameDrawer<D>,
		F: FrameDrawer<D>,
{
	type Event = ();
	type Model = (f32, bool);
	fn behavior(&self, ctx: &Context<D>, _state: &mut UiState, model: &mut Self::Model) -> Self::Event {
		let rect = ctx.rect();
		let fill_rect = if model.1 {
			rect.h(rect.dy() * model.0)
		} else {
			rect.w(rect.dx() * model.0)
		};
		self.background.draw_frame(ctx.draw(), rect);
		self.fill.draw_frame(ctx.draw(), fill_rect);
	}
}
