use math::*;
use draw;

use std::ptr::NonNull;

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

pub enum BrushOwner<C> {
    Mask(Vec<bool>),
    Blit(Vec<C>),
}

impl<C> BrushOwner<C> {
    pub fn get(&self) -> Brush<C> {
        match self {
            BrushOwner::Mask(data) => Brush::Mask(data.as_slice().into()),
            BrushOwner::Blit(data) => Brush::Blit(data.as_slice().into()),
        }
    }
}

pub enum Brush<C> {
    Mask(NonNull<[bool]>),
    Blit(NonNull<[C]>),
}

pub trait PreviewContext<N, C>: draw::CanvasRead<C, N> + draw::CanvasWrite<C, N> + draw::Bounded<N>
    where N: BaseIntExt, C: Copy + Clone + Eq
{
    fn brush(&self) -> (Brush<C>, Rect<N>);
    fn paint_brush(&mut self, p: Point2<N>, color: C) {
        let r = {
            let (brush, r) = self.brush();
            let r = r.shift_x_y(p.x, p.y);
            match brush {
                Brush::Mask(brush) => self.mask(r, unsafe { brush.as_ref() }, color),
                Brush::Blit(brush) => self.blit(r, unsafe { brush.as_ref() }),
            };
            r
        };
    }
}

pub trait Context<N, C>: draw::CanvasRead<C, N> + draw::CanvasWrite<C, N> + draw::Bounded<N>
    where N: BaseIntExt, C: Copy + Clone + Eq
{
    fn start(&mut self) -> C;
    fn commit(&mut self);
    fn rollback(&mut self);
    fn sync(&mut self);
    fn change_color(&mut self, color: C);

    fn brush(&self) -> (Brush<C>, Rect<N>);

    fn paint_brush(&mut self, p: Point2<N>, color: C) {
        let r = {
            let (brush, r) = self.brush();
            let r = r.shift_x_y(p.x, p.y);
            match brush {
                Brush::Mask(brush) => self.mask(r, unsafe { brush.as_ref() }, color),
                Brush::Blit(brush) => self.blit(r, unsafe { brush.as_ref() }),
            };
            r
        };
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
    /*
    fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx) {
        match input {
            Input::Move(p) => self.movement(p, ctx),
            Input::Special(on) => self.special(on, ctx),
            Input::Press(p) => self.press(p, ctx),
            Input::Release(p) => self.release(p, ctx),
            Input::Cancel => self.cancel(ctx),
        }
    }
    */

    fn press<Ctx: Context<N, C>>(&mut self, _p: Point2<N>, ctx: &mut Ctx) {}
    fn release<Ctx: Context<N, C>>(&mut self, _p: Point2<N>, ctx: &mut Ctx) {}
    fn movement<Ctx: Context<N, C>>(&mut self, _p: Point2<N>, ctx: &mut Ctx) {}
    // shift ?
    fn special<Ctx: Context<N, C>>(&mut self, _on: bool, ctx: &mut Ctx) {}
    // press ESC
    fn cancel<Ctx: Context<N, C>>(&mut self, ctx: &mut Ctx) {}

    fn preview<Ctx: PreviewContext<N, C>>(&self, mouse: Point2<N>, ctx: &mut Ctx) {}
}
