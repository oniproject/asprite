use super::*;

pub struct Checkbox<P, H, C, U> {
    pub normal_bg: N,
    pub hovered_bg: H,
    pub pressed_bg: P,
    pub checked: C,
    pub unchecked: U,
}

impl<'a, D, P, H, N> Component<Context<'a, D>, UiState> for Toggle<P, H, N, C, U>
    where
        D: ?Sized + Graphics + 'a,
        P: Painter<D>,
        H: Painter<D>,
        N: Painter<D>,
        C: Painter<D>,
        U: Painter<D>,
{
    type Event = bool;
    type Model = bool;
    fn behavior(&self, ctx: &Context<D>, state: &mut UiState, checked: &mut Self::Model) -> Self::Event {
        let id = ctx.reserve_widget_id();
        let event = if state.active_widget == Some(id) {
            ctx.set_hovered();
            self.pressed.paint(ctx.draw(), ctx.rect());
            let event = ctx.was_released();
            if event || !ctx.is_cursor_hovering() {
                state.active_widget = None;
            }
            event
        } else if ctx.is_cursor_hovering() && state.active_widget == None {
            ctx.set_hovered();
            self.hovered.paint(ctx.draw(), ctx.rect());
            if ctx.was_pressed() {
                state.active_widget = Some(id);
            }
            false
        } else {
            self.normal.paint(ctx.draw(), ctx.rect());
            false
        }

        if event { *checked = !*checked; }
        if checked {
            self.checked.paint(ctx.draw(), ctx.rect());
        } else {
            self.unchecked.paint(ctx.draw(), ctx.rect());
        }
        event
    }
}
