#![allow(dead_code)]

use common::*;
pub mod freehand;

#[derive(Clone, Debug)]
pub enum Input {
	Press(Point<i16>),
	Release(Point<i16>),
	Move(Point<i16>),
	Cancel, // press ESC
}

pub trait Context {
	fn start(&mut self) -> u8;
	fn commit(&mut self);
	fn rollback(&mut self);
	fn sync(&mut self);

	fn brush(&mut self, Point<i16>, u8);
}

pub trait Tool {
	fn run<C: Context>(&mut self, input: Input, ctx: &mut C);
}