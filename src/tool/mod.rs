#![allow(dead_code)]

use common::*;

mod freehand;
mod rectangle;
mod bucket;

pub use self::freehand::Freehand;
pub use self::rectangle::Rectangle;
pub use self::bucket::Bucket;

#[derive(Clone, Debug)]
pub enum Input<N: Signed> {
	Press(Point<N>),
	Release(Point<N>),
	Move(Point<N>),
	Special(bool),
	Cancel, // press ESC
}

pub trait Context<N: Signed, C: Copy + PartialEq>: Image<N, C> {
	fn start(&mut self) -> C;
	fn commit(&mut self);
	fn rollback(&mut self);
	fn sync(&mut self);
}

pub trait Tool<N: Signed, C: Copy + PartialEq> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx);
}