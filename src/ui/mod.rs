pub mod layout;
pub mod components;

mod guard;

mod id;
mod transform;
mod context;
mod graphics;
mod mouse;
mod ninepatch;
mod frame_drawer;

pub use self::transform::*;

pub use self::guard::*;
pub use self::ninepatch::*;

pub use self::mouse::{Mouse, MouseEvent};
pub use self::graphics::Graphics;
pub use self::frame_drawer::{NoDrawer, FrameDrawer, ColorDrawer, TextureDrawer};

pub use self::context::{Context, ContextBuilder, Events};
pub use self::id::{Id, IdGenerator, Generator};

pub use self::layout::flow::{
    Flow,
    LayoutIter,
    layout,
    measure,
};

pub use self::components::{
    ActiveWidget,
    UiState,
};

pub use self::components::button::Button;
pub use self::components::progress::Progress;
pub use self::components::toggle::Toggle;
pub use self::components::slider::{Slider, SliderModel};

pub use self::components::menubar::{
    Item,
    ItemStyle,
    Menu,
    MenuEvent,
    MenuBar,
    MenuBarModel,
};


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
