pub mod button;
pub mod slider;
pub mod toggle;
pub mod progress;
pub mod menubar;
pub mod menu;

use super::{Context, Events, Id, Painter, ColorDrawer, TextureDrawer, Graphics};

pub trait Component<C, S> {
    type Event;
    type Model;
    fn behavior(&self, ctx: &C, state: &mut S, model: &mut Self::Model) -> Self::Event;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiState {
    pub(in ui) active_widget: Option<Id>,
}

impl UiState {
    pub const fn new() -> Self {
        Self { active_widget: None }
    }

    #[inline(always)]
    pub(in ui) fn active_widget(&self) -> Option<Id> {
        self.active_widget
    }
    #[inline(always)]
    pub(in ui) fn active_widget_mut(&mut self) -> &mut Option<Id> {
        &mut self.active_widget
    }
}
