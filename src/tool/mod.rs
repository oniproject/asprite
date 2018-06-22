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

pub type BrushOwned = Vec<bool>;
pub type Brush<'a> = &'a [bool];

unsafe fn get_brush(data: Brush) -> NonNull<[bool]> {
    data.as_ref().into()
}

pub trait PreviewContext<N, C>: Sized + draw::CanvasRead<C, N> + draw::CanvasWrite<C, N> + draw::Bounded<N>
    where N: BaseIntExt, C: Copy + Clone + Eq
{
    fn color(&self) -> C;
    fn brush(&self) -> (Brush, Rect<N>);
    fn paint_brush(&mut self, p: Point2<N>, color: C) {
        let (brush, r) = unsafe {
            let (brush, r) = self.brush();
            (get_brush(brush), r.shift_x_y(p.x, p.y))
        };
        unsafe { self.mask(r, brush.as_ref(), color); }
    }
}

pub trait Context<N, C>: Sized + draw::CanvasRead<C, N> + draw::CanvasWrite<C, N> + draw::Bounded<N>
    where N: BaseIntExt, C: Copy + Clone + Eq
{
    fn start(&mut self);
    fn commit(&mut self);
    fn rollback(&mut self);
    fn sync(&mut self);

    fn color(&self) -> C;
    fn change_color(&mut self, color: C);

    fn brush(&self) -> (Brush, Rect<N>);

    fn paint_brush(&mut self, p: Point2<N>, color: C) {
        let (brush, r) = unsafe {
            let (brush, r) = self.brush();
            (get_brush(brush), r.shift_x_y(p.x, p.y))
        };
        unsafe { self.mask(r, brush.as_ref(), color); }
        self.update(r.pad(-N::one()-N::one()-N::one()));
    }

    fn update(&mut self, r: Rect<N>);
    fn update_point(&mut self, p: Point2<N>) {
        let one = N::one();
        let r = Rect::from_coords_and_size(p.x, p.y, one, one);
        self.update(r);
    }
}

pub trait Tool<N, C>
    where N: BaseIntExt, C: Copy + Clone + Eq
{
    fn press   <Ctx: Context<N, C>>(&mut self, _p: Point2<N>, ctx: &mut Ctx) {}
    fn release <Ctx: Context<N, C>>(&mut self, _p: Point2<N>, ctx: &mut Ctx) {}
    fn movement<Ctx: Context<N, C>>(&mut self, _p: Point2<N>, ctx: &mut Ctx) {}

    // shift ?
    fn special<Ctx: Context<N, C>>(&mut self, _on: bool, ctx: &mut Ctx) {}
    // press ESC
    fn cancel<Ctx: Context<N, C>>(&mut self, ctx: &mut Ctx) {}

    fn preview<Ctx: PreviewContext<N, C>>(&self, mouse: Point2<N>, ctx: &mut Ctx) {}
}
