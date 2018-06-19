use sdl2::gfx::primitives::DrawRenderer;

use math::Rect;
use draw::{Palette, Bounded, CanvasRead, CanvasWrite};
use tool::{Editor, Brush, PreviewContext, Context};
use render::TextureCanvas;

pub struct Prev<'a> {
    pub canvas: &'a mut TextureCanvas,
    pub rect: Rect<i32>,
    pub palette: &'a Palette<u32>,
    pub editor: &'a Editor,
}

impl<'a> Bounded<i32> for Prev<'a> {
    #[inline(always)]
    fn bounds(&self) -> Rect<i32> {
        self.rect
    }
}

impl<'a> CanvasWrite<u8, i32> for Prev<'a> {
    #[inline(always)]
    unsafe fn set_pixel_unchecked(&mut self, x: i32, y: i32, color: u8) {
        let c = self.palette[color].to_be();
        self.canvas.pixel(x as i16, y as i16, c).unwrap()
    }
}

impl<'a> CanvasRead<u8, i32> for Prev<'a> {
    #[inline(always)]
    unsafe fn get_pixel_unchecked(&self, x: i32, y: i32) -> u8 {
        self.editor.get_pixel_unchecked(x, y)
    }
}

impl<'a> PreviewContext<i32, u8> for Prev<'a> {
    fn brush(&self) -> (Brush<u8>, Rect<i32>) { self.editor.brush() }
}
