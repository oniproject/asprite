#![allow(dead_code)]

pub mod gradient;
mod shape;
mod bresenham;

mod common;
mod canvas;
mod palette;
mod frame;
mod scanline;
mod view_read;
mod view_write;
mod uniform;

pub use self::shape::{
    Shape,
    round,
    square,
    sieve_round,
    sieve_square,
    plus,
    slash,
    antislash,
    horizontal_bar,
    vertical_bar,
    cross,
    diamond,
};
pub use self::view_read::ViewRead;
pub use self::view_write::ViewWrite;
pub use self::uniform::Uniform;

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
