pub mod button;
pub mod slider;
pub mod toggle;
pub mod progress;
pub mod menubar;

use super::{Context, Component, Events, Id, FrameDrawer, ColorDrawer, TextureDrawer, Graphics, MouseEvent};
use super::transform::*;

pub trait ActiveWidget {
    fn active_widget(&self) -> Option<Id>;
    fn active_widget_mut(&mut self) -> &mut Option<Id>;

    #[inline(always)]
    fn is_active(&self, id: Id) -> bool {
        self.active_widget() == Some(id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiState {
    pub(in ui) active_widget: Option<Id>,
}

impl UiState {
    pub const fn new() -> Self {
        Self { active_widget: None }
    }
}

impl ActiveWidget for UiState {
    #[inline(always)]
    fn active_widget(&self) -> Option<Id> {
        self.active_widget
    }
    #[inline(always)]
    fn active_widget_mut(&mut self) -> &mut Option<Id> {
        &mut self.active_widget
    }
}
