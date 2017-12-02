use super::*;

pub struct Button<P, H, N> {
	pub pressed: P,
	pub hovered: H,
	pub normal: N,
}

impl<'a, D, P, H, N> Component<Context<'a, D>, UiState> for Button<P, H, N>
	where
		D: ?Sized + Graphics + 'a,
		P: FrameDrawer<D>,
		H: FrameDrawer<D>,
		N: FrameDrawer<D>,
{
	type Event = bool;
	type Model = ();
	fn behavior(&self, ctx: &Context<D>, state: &mut UiState, _: &mut Self::Model) -> Self::Event {
		let id = ctx.reserve_widget_id();
		if state.active_widget == Some(id) {
			ctx.set_hovered();
			self.pressed.draw_frame(ctx.draw(), ctx.rect());
			let event = ctx.was_released();
			if event || !ctx.is_cursor_hovering() {
				state.active_widget = None;
			}
			event
		} else if ctx.is_cursor_hovering() && state.active_widget == None {
			ctx.set_hovered();
			self.hovered.draw_frame(ctx.draw(), ctx.rect());
			if ctx.was_pressed() {
				state.active_widget = Some(id);
			}
			false
		} else {
			self.normal.draw_frame(ctx.draw(), ctx.rect());
			false
		}
	}
}
