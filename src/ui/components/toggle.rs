use super::*;
use super::button::Button;

pub struct Toggle<P, H, N> {
    pub checked: Button<P, H, N>,
    pub unchecked: Button<P, H, N>,
}

impl<'a, D, P, H, N> Component<Context<'a, D>, UiState> for Toggle<P, H, N>
    where
        D: ?Sized + Graphics + 'a,
        P: FrameDrawer<D>,
        H: FrameDrawer<D>,
        N: FrameDrawer<D>,
{
    type Event = bool;
    type Model = bool;
    fn behavior(&self, ctx: &Context<D>, state: &mut UiState, checked: &mut Self::Model) -> Self::Event {
        let btn = if *checked { &self.checked } else { &self.unchecked };
        let event = btn.behavior(ctx, state, &mut ());
        if event { *checked = !*checked; }
        event
    }
}
