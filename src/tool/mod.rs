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
	fn commit(&mut self);
	fn rollback(&mut self);

	fn brush(&mut self, Point<i16>);
}

pub struct SimpleContext {
	pub grid: Vec<u8>, 
	pub size: Point<i16>,
	pub pos: Point<i16>,
	pub mouse: Point<i16>,
	pub zoom: i16,
	pub color: u8,
}

impl SimpleContext {
	pub fn set_mouse(&mut self, x: i32, y: i32) -> Point<i16> {
		let x = (x as i16 - self.pos.x) / self.zoom;
		let y = (y as i16 - self.pos.y) / self.zoom;
		self.mouse = Point::new(x, y);
		self.mouse
	}
}

impl Context for SimpleContext {
	fn commit(&mut self) {}
	fn rollback(&mut self) {}

	fn brush(&mut self, p: Point<i16>) {
		let x = p.x >= 0 && p.x < self.size.x;
		let y = p.y >= 0 && p.y < self.size.y;
		if x && y {
			let idx = p.x + p.y * self.size.x;
			self.grid[idx as usize] = self.color;
		}
	}
}

pub trait Tool {
	fn run<C: Context>(&mut self, input: Input, ctx: &mut C);
}