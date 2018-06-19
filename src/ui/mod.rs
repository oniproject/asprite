pub mod layout;
pub mod components;

mod id;
mod context;
mod mouse;
mod painter;

pub use self::mouse::{Mouse, Events};
pub use self::painter::{
    Graphics,
    Painter,
    NoDrawer, ColorDrawer, TextureDrawer,
    NinePatch,
};

pub use self::context::{Context, ContextBuilder};
pub use self::id::{Id, IdGenerator, Generator};

pub use self::layout::flow::{
    Flow,
    layout,
    measure,
};

pub use self::components::{UiState, Component};
pub use self::components::button::Button;
pub use self::components::progress::Progress;
pub use self::components::toggle::Toggle;
pub use self::components::slider::{Slider, SliderModel};
pub use self::components::menubar::{MenuBar,MenuBarModel};
pub use self::components::menu::{
    Item,
    ItemStyle,
    Menu,
    MenuEvent,
};

pub type SimpleButton<D> = Button<D, D, D>;
pub type ColorButton<D> = SimpleButton<ColorDrawer<D>>;
pub type TextureButton<D> = SimpleButton<TextureDrawer<D>>;

pub type SimpleToggle<D> = Toggle<D, D, D>;
pub type ColorToggle<D> = SimpleToggle<ColorDrawer<D>>;
