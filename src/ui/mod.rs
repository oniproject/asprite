mod guard;

mod id;
mod transform;
mod flow;
mod context;
mod graphics;
mod mouse;
mod ninepatch;

mod frame_drawer;

pub use self::transform::*;

pub use self::flow::*;
pub use self::guard::*;
pub use self::context::*;
pub use self::id::*;
pub use self::graphics::*;
pub use self::mouse::*;
pub use self::ninepatch::*;
pub use self::frame_drawer::*;

mod button;
mod toggle;
mod progress;
mod slider;

pub mod menubar;

pub use self::button::*;
pub use self::toggle::*;
pub use self::progress::*;
pub use self::slider::*;

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
    active_widget: Option<Id>,
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

pub type SimpleButton<D> = Button<D, D, D>;
pub type ColorButton<D> = SimpleButton<ColorDrawer<D>>;
pub type TextureButton<D> = SimpleButton<TextureDrawer<D>>;

pub type SimpleToggle<D> = Toggle<D, D, D>;
pub type ColorToggle<D> = SimpleToggle<ColorDrawer<D>>;

pub trait Component<C, S> {
    type Event;
    type Model;
    fn behavior(&self, ctx: &C, state: &mut S, model: &mut Self::Model) -> Self::Event;
}
