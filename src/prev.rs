use math::Rect;
use draw::{Bounded, CanvasRead, CanvasWrite};
use tool::{Editor, Brush, PreviewContext};

pub struct Prev<'a> {
    pub ptr: *mut u8,
    pub rect: Rect<i32>,
    pub editor: &'a Editor,
}

impl<'a> Bounded<i32> for Prev<'a> {
    #[inline(always)]
    fn bounds(&self) -> Rect<i32> { self.rect }
}

impl<'a> CanvasWrite<u8, i32> for Prev<'a> {
    #[inline(always)]
    unsafe fn set_unchecked(&mut self, x: i32, y: i32, color: u8) {
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
    unsafe fn at_unchecked(&self, x: i32, y: i32) -> u8 {
        self.editor.at_unchecked(x, y)
    }
}

impl<'a> PreviewContext<i32, u8> for Prev<'a> {
    fn color(&self) -> u8 { self.editor.color() }
    fn brush(&self) -> (Brush, Rect<i32>) { self.editor.brush() }
}
