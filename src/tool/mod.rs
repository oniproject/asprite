#![allow(dead_code)]

use common::*;
pub mod freehand;

#[derive(Clone, Debug)]
pub enum Input<N: Signed> {
	Press(Point<N>),
	Release(Point<N>),
	Move(Point<N>),
	Special(bool),
	Cancel, // press ESC
}

pub trait Context<N: Signed, C: Copy>: Image<N, C> {
	fn start(&mut self) -> C;
	fn commit(&mut self);
	fn rollback(&mut self);
	fn sync(&mut self);
}

pub trait Tool<N: Signed, C: Copy> {
	fn run<Ctx: Context<N, C>>(&mut self, input: Input<N>, ctx: &mut Ctx);
}