use math::*;
use draw;

use std::iter::Step;

mod freehand;
mod primitive;
mod bucket;
mod eye_dropper;

pub use self::freehand::Freehand;
pub use self::primitive::{Primitive, PrimitiveMode};
pub use self::bucket::Bucket;
pub use self::eye_dropper::EyeDropper;

#[derive(Clone, Debug)]
pub enum Input<N: BaseNum> {
    Press(Point2<N>),
    Release(Point2<N>),
    Move(Point2<N>),
    Special(bool),
    Cancel, // press ESC
}

pub trait Context<N: BaseNumExt + Step, C: Copy + Clone + Eq>: draw::Canvas<C, N> {
    fn start(&mut self) -> C;
    fn commit(&mut self);
    fn rollback(&mut self);
    fn sync(&mut self);
    fn change_color(&mut self, C);
    fn paint_brush(&mut self, p: Point2<N>, C);

    fn update(&mut self, r: Rect<N>);
    fn update_point(&mut self, p: Point2<N>) {
        let one = N::one();
        let r = Rect::from_coords_and_size(p.x, p.y, one, one);
        self.update(r);
    }
}

pub trait Tool<N: BaseNumExt + Step, C: Copy + Clone + Eq> {
    fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx);
    fn preview<Ctx: Context<N, C>>(&mut self, x: N, y: N, ctx: &mut Ctx) {}
}
