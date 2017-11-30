use super::*;

pub struct Toggle<P, H, N> {
	pub checked: Button<P, H, N>,
	pub unchecked: Button<P, H, N>,
}

impl<D, P, H, N> Component<D> for Toggle<P, H, N>
	where
		D: ?Sized + Graphics,
		P: FrameDrawer<D>,
		H: FrameDrawer<D>,
		N: FrameDrawer<D>,
{
	type Event = ();
	type Model = bool;
	fn behavior(&self, ctx: &Context<D>, state: &mut UiState, checked: &mut Self::Model) -> Self::Event {
		if *checked {
			if self.checked.behavior(ctx, state, &mut ()) {
				*checked = false;
			}
		} else {
			if self.unchecked.behavior(ctx, state, &mut ()) {
				*checked = true;
			}
		}
	}
}
