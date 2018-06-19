use math::*;
use super::{Id, UiState};

#[derive(Clone, Copy, Debug)]
pub struct Mouse {
    pub cursor: Point2<f32>,
    pub pressed: bool,
    pub released: bool,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            cursor: Point2::new(-14.0, -88.0),
            pressed: false,
            released: false,
        }
    }

    pub fn cleanup(&mut self) {
        self.pressed = false;
        self.released = false;
    }
}

pub trait Events {
    fn rect(&self) -> Rect<f32>;

    fn cursor(&self) -> Point2<f32>;
    fn was_pressed(&self) -> bool;
    fn was_released(&self) -> bool;
    fn is_cursor_in_rect(&self, r: Rect<f32>) -> bool {
        r.contains(self.cursor())
    }

    fn onhover<E, L>(&self, id: Id, rect: Rect<f32>, state: &mut UiState, enter: E, leave: L)
        where E: FnOnce(), L: FnOnce(),
    {
        if self.is_cursor_in_rect(rect) {
            if state.active_widget == None {
                state.active_widget = Some(id);
                enter();
            }
        } else if state.active_widget == Some(id) {
            state.active_widget = None;
            leave();
        }
    }
    fn onclick<F: FnOnce()>(&self, id: Id, rect: Rect<f32>, state: &mut UiState, f: F) {
        let hovered = self.is_cursor_in_rect(rect);
        if hovered {
            if state.active_widget == None && self.was_pressed() {
                state.active_widget = Some(id);
            }
            if state.active_widget == Some(id) && self.was_released() {
                state.active_widget = None;
                f()
            }
        } else {
            if state.active_widget == Some(id) {
                state.active_widget = None;
            }
        }
    }
}
