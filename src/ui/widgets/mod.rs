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
use super::{Graphics, Mouse};

pub use self::layout::*;
pub use self::button::Button;
pub use self::label::Label;
pub use self::window::Window;

pub fn example<G: Graphics<i16, u32>>() -> window::Window<i16, u32, G> {
	let r = Rect::with_size(800, 100, 420, 500);
	let mut win = Window::new(r);

	for i in 0..5 {
		let btn = Button::new(format!("fuck #{}", i), move |_| {
			println!("fuck u #{}", i);
		});
		btn.wh(60, 20);
		win.add(Rc::new(btn));
	}

	for i in 0..5 {
		let text = Label::new(format!("fuck #{}", i));
		text.wh(60, 20);
		win.add(Rc::new(text));
	}

	layout_vertical(win.rect.get(), &win.widgets);

	win
}

pub trait Bounds<N: Signed> {
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

pub trait Widget<N: Signed, C: Copy + 'static, G: Graphics<N, C>>: Bounds<N> + Any {
	fn paint(&self, ctx: &mut G, focused: bool);
	fn event(&self, event: Event<N>, focused: bool, redraw: &Cell<bool>) -> bool;
}

#[derive(Copy, Clone, Debug)]
pub enum Event<N: Signed> {
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