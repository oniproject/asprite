use std::cell::RefCell;
use std::any::Any;
use std::rc::Rc;

use shape;

pub trait Layer: Any {
	fn meta(&mut self) -> &mut Meta;
}

#[derive(Debug)]
pub struct Meta {
	pub name: String,
	pub visible: bool,
	pub locked: bool,
	pub opacity: f32,
	pub offset: (f32, f32),
}

pub struct Group {
	pub layers: Vec<Rc<RefCell<Layer>>>,
	pub meta: Meta,
}

pub struct Tiled {
	pub data: Vec<u32>,
	pub meta: Meta,
}

pub struct Shapes {
	pub data: Vec<shape::Shape>,
	pub meta: Meta,
}

impl Layer for Group {
	fn meta(&mut self) -> &mut Meta { &mut self.meta }
}

impl Layer for Tiled {
	fn meta(&mut self) -> &mut Meta { &mut self.meta }
}

impl Layer for Shapes {
	fn meta(&mut self) -> &mut Meta { &mut self.meta }
}
