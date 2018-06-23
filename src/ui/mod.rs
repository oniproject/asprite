#![allow(dead_code)]

mod layout;
mod components;

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
    Axis,
    Flow,
    layout,
    measure,
};

pub use self::components::{
    UiState,
    Component,
    button::Button,
    progress::Progress,
    toggle::Toggle,
    slider::{Slider, SliderModel},
    menubar::{MenuBar,MenuBarModel},
    menu::{
        Item,
        ItemStyle,
        Menu,
        MenuEvent,
    },
};

pub type SimpleTransparentButton<D> = Button<D, D, NoDrawer>;
pub type ColorTransparentButton<D> = Button<ColorDrawer<D>, ColorDrawer<D>, NoDrawer>;

pub type SimpleButton<D> = Button<D, D, D>;
pub type ColorButton<D> = SimpleButton<ColorDrawer<D>>;
pub type TextureButton<D> = SimpleButton<TextureDrawer<D>>;

pub type SimpleTransparentToggle<D> = Toggle<D, D, NoDrawer>;
pub type ColorTransparentToggle<D> = Toggle<ColorDrawer<D>, ColorDrawer<D>, NoDrawer>;

pub type SimpleToggle<D> = Toggle<D, D, D>;
pub type ColorToggle<D> = SimpleToggle<ColorDrawer<D>>;
