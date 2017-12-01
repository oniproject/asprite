use super::*;
use math::*;

#[derive(Clone, Copy)]
pub struct Progress<BG, F> {
	pub background: BG,
	pub fill: F,
	pub axis: Axis,
}

impl<D, BG, F> Component<D> for Progress<BG, F>
	where
		D: ?Sized + Graphics,
		BG: FrameDrawer<D>,
		F: FrameDrawer<D>,
{
	type Event = ();
	type Model = f32;
	fn behavior(&self, ctx: &Context<D>, _state: &mut UiState, model: &mut Self::Model) -> Self::Event {
		let rect = ctx.rect();
		let fill_rect = match self.axis {
			Axis::Horizontal => rect.w(rect.dx() * *model),
			Axis::Vertical =>   rect.h(rect.dy() * *model),
		};
		self.background.draw_frame(ctx.draw(), rect);
		self.fill.draw_frame(ctx.draw(), fill_rect);
	}
}
