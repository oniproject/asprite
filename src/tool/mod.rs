use math::*;
use draw;

use std::ptr::NonNull;

mod brush;

mod editor;
mod receiver;

mod freehand;
mod primitive;
mod bucket;
mod eye_dropper;

pub use self::receiver::{Receiver, Layer};
pub use self::editor::Editor;

pub use self::freehand::Freehand;
pub use self::primitive::{Primitive, PrimitiveMode};
pub use self::bucket::Bucket;
pub use self::eye_dropper::EyeDropper;

pub type Brush<'a> = &'a [bool];

pub trait PreviewContext<N, C>: Sized + draw::CanvasRead<C, N> + draw::CanvasWrite<C, N> + draw::Bounded<N>
    where N: BaseIntExt, C: Copy + Eq
{
    fn color(&self) -> C;
    fn brush(&self) -> (Brush, Rect<N>);
    fn paint_brush(&mut self, p: Point2<N>, color: C) {
        // FIXME: ugly hack around borrow checker
        unsafe {
            let br;
            let brush: NonNull<[bool]> = {
                let (brush, r) = self.brush();
                br = r.shift_x_y(p.x, p.y);
                brush.as_ref().into()
            };
            if let Some(r) = self.intersect(br) {
                draw::mask(r, br, brush.as_ref(), |x, y| self.set_unchecked(x, y, color))
            }
        }
    }
}

pub trait Context<N, C>: Sized + draw::CanvasRead<C, N> + draw::CanvasWrite<C, N> + draw::Bounded<N> + PreviewContext<N, C>
    where N: BaseIntExt, C: Copy + Eq
{
    fn start(&mut self);
    fn commit(&mut self);
    fn rollback(&mut self);
    fn sync(&mut self);
    fn change_color(&mut self, color: C);
}

pub trait Tool<N, C>
    where N: BaseIntExt, C: Copy + Eq
{
    fn press   <Ctx: Context<N, C>>(&mut self, _p: Point2<N>, _ctx: &mut Ctx) {}
    fn release <Ctx: Context<N, C>>(&mut self, _p: Point2<N>, _ctx: &mut Ctx) {}
    fn movement<Ctx: Context<N, C>>(&mut self, _p: Point2<N>, _ctx: &mut Ctx) {}

    // shift ?
    fn special<Ctx: Context<N, C>>(&mut self, _on: bool, _ctx: &mut Ctx) {}
    // press ESC
    fn cancel<Ctx: Context<N, C>>(&mut self, _ctx: &mut Ctx) {}

    fn preview<Ctx: PreviewContext<N, C>>(&self, _mouse: Point2<N>, _ctx: &mut Ctx) {}
}
