#![allow(dead_code)]
mod theme;
mod check_set;
mod window;
mod button;
mod label;
mod layout;

use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

use common::*;
use super::Mouse;

pub use self::layout::*;
pub use self::button::Button;
pub use self::label::Label;
pub use self::window::Root;

pub trait Graphics<N: SignedInt, C: Copy + 'static> {
	fn render_text_center(&mut self, r: Rect<N>, color: C, s: &str);
	fn render_rect(&mut self, r: Rect<N>, color: C);
	fn render_border(&mut self, r: Rect<N>, color: C);
}

pub fn example() -> Root<i16, u32> {
	let r = Rect::with_size(800, 100, 420, 500);
	let mut root = Root::new(r);

	let r = Rect::with_size(0, 0, 420, 500);
	let mut list = Flow::vertical(r);
	for i in 0..5 {
		let btn = Rc::new(Button::new(format!("fuck #{}", i), move |_| {
			println!("fuck u #{}", i);
		}));
		btn.wh(60, 20);
		list.add(btn);
	}

	for i in 0..5 {
		let text = Rc::new(Label::new(format!("fuck #{}", i)));
		text.wh(60, 20);
		list.add(text);
	}

	root.add(Rc::new(list));
	root.measure();
	root.layout();
	root
}

pub trait Layout {
	fn layout(&self) {}
	fn layout_data(&self) -> Option<&Any> {
		None
	}
}

pub trait Measured<N: SignedInt> {
	fn measured_size(&self) -> &Cell<Point<N>>;
	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		self.measured_size().set(Point::new(N::zero(), N::zero()))
	}
}

pub trait Bounds<N: SignedInt> {
	fn bounds(&self) -> &Cell<Rect<N>>;

	fn wh(&self, w: N, h: N) {
		let r = self.bounds().get();
		self.bounds().set(r.wh(w, h));
	} 

	fn size(&self) -> (N, N) {
		let r = self.bounds().get();
		(r.dx(), r.dy())
	} 

	fn pos(&self) -> (N, N) {
		let r = self.bounds().get();
		(r.min.x, r.min.y)
	} 
}

pub trait Widget<N: SignedInt, C: Copy + 'static>: Bounds<N> + Measured<N> + Layout + Any {
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, focused: bool);
	fn event(&self, event: Event<N>, origin: Point<N>, focused: bool, redraw: &Cell<bool>) -> bool;
}

#[derive(Copy, Clone, Debug)]
pub enum Event<N: SignedInt> {
	Init,
	Mouse {
		point: Point<N>,
		left: bool,
		middle: bool,
		right: bool,
	},
	Scroll {
		x: N,
		y: N,
	},
	Text {
		c: char,
	},
	Enter,
	Backspace,
	Delete,
	Home,
	End,
	UpArrow,
	DownArrow,
	LeftArrow,
	RightArrow,
	Resize {
		width: u32,
		height: u32,
	},
	Unknown,
}