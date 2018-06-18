pub mod gradient;

mod canvas;
mod palette;
mod layer;
mod frame;
mod sprite;

pub use self::canvas::{
    Bounded,
    CanvasRead,
    CanvasWrite,
    CanvasFill,
    blit,
    draw_line,
};
pub use self::palette::Palette;
pub use self::frame::Frame;
pub use self::layer::Layer;
pub use self::sprite::Sprite;
