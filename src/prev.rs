use sdl2::gfx::primitives::DrawRenderer;

use math::Rect;
use draw::{Palette, Bounded, CanvasRead, CanvasWrite};
use tool::{Editor, Brush, PreviewContext, Context};
use render::TextureCanvas;

pub struct Prev<'a> {
    //pub canvas: &'a mut TextureCanvas,
    pub ptr: *mut u8,
    pub rect: Rect<i32>,
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
        let m = self.editor.image.as_receiver();
        let c = if self.editor.transparent() == Some(color) {
            0
        } else {
            m.palette[color].to_le()
        };
        let ptr = self.ptr.add((m.width * y as usize + x as usize) * 4);
        *ptr.add(0) = ( c        & 0xFF) as u8;
        *ptr.add(1) = ((c >>  8) & 0xFF) as u8;
        *ptr.add(2) = ((c >> 16) & 0xFF) as u8;
        *ptr.add(3) = ((c >> 24) & 0xFF) as u8;
    }
}

impl<'a> CanvasRead<u8, i32> for Prev<'a> {
    #[inline(always)]
    unsafe fn get_pixel_unchecked(&self, x: i32, y: i32) -> u8 {
        self.editor.get_pixel_unchecked(x, y)
    }
}

impl<'a> PreviewContext<i32, u8> for Prev<'a> {
    fn color(&self) -> u8 { self.editor.image.as_receiver().color }
    fn brush(&self) -> (Brush, Rect<i32>) { self.editor.brush() }
}
