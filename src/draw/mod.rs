#![allow(dead_code)]

pub mod gradient;
pub mod shape;
mod bresenham;

mod common;
mod canvas;
mod palette;
mod frame;
mod scanline;
mod view;

pub use self::shape::Shape;

pub use self::view::{View, ViewMut, Uniform};

pub use self::bresenham::Bresenham;
pub use self::scanline::ScanlineFill;

pub use self::common::{
    blit,
    mask,
    draw_line,
    draw_ellipse,
    fill_rect,
};

pub use self::canvas::{
    Bounded,
    CanvasRead,
    CanvasWrite,
};
pub use self::palette::Palette;
pub use self::frame::Frame;
