#![allow(dead_code)]

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use common::*;
use super::*;

pub trait CheckSet<T> {
	fn check_set(&self, v: T) -> bool;
}

impl<T: Copy> CheckSet<T> for Cell<T> where T: PartialOrd {
	fn check_set(&self, value: T) -> bool {
		if value != self.get() {
			self.set(value);
			true
		} else {
			false
		}
	}
}

pub type Color = u32;

const BLACK: Color = 0x000000_FF;
const SELECT_BLUE: Color = 0x5294E2_FF;
const BORDER_GREY: Color = 0xCFD6E6_FF;
const WINDOW_GREY: Color = 0xF5F6F7_FF;
const BUTTON_WHITE: Color = 0xFBFBFC_FF;
const WHITE: Color = 0xFFFFFF_FF;

pub static WINDOW_BACKGROUND: Color = WINDOW_GREY;

pub static LABEL_BACKGROUND: Color = WINDOW_GREY;
pub static LABEL_BORDER: Color = BORDER_GREY;
pub static LABEL_FOREGROUND: Color = BLACK;

pub static BUTTON_BACKGROUND: Color = BUTTON_WHITE;
pub static BUTTON_BG_SELECTION: Color = SELECT_BLUE;
pub static BUTTON_BORDER: Color = BORDER_GREY;
pub static BUTTON_FOREGROUND: Color = BLACK;
pub static BUTTON_FG_SELECTION: Color = WHITE;

pub static ITEM_BACKGROUND: Color = WHITE;
pub static ITEM_BORDER: Color = BORDER_GREY;
pub static ITEM_FOREGROUND: Color = BLACK;
pub static ITEM_SELECTION: Color = SELECT_BLUE;

pub static TEXT_BACKGROUND: Color = WHITE;
pub static TEXT_BORDER: Color = BORDER_GREY;
pub static TEXT_FOREGROUND: Color = BLACK;
pub static TEXT_SELECTION: Color = SELECT_BLUE;

pub struct Window<N: Signed, C: Copy, G: Graphics<N, C>> {
	pub widgets: Vec<Rc<Widget<N, C, G>>>,
	pub focus: Cell<usize>,
	pub rect: Cell<Rect<N>>,
	pub redraw: Cell<bool>,
}

impl<N: Signed, C: Copy, G: Graphics<N, C>> Window<N, C, G> {
	pub fn new(&self, rect: Rect<N>) -> Self {
		Self {
			widgets: Vec::new(),
			focus: Cell::new(usize::max_value()),
			rect: Cell::new(rect),
			redraw: Cell::new(true),
		}
	}

	pub fn add(&mut self, w: Rc<Widget<N, C, G>>) {
		self.widgets.push(w);
	}

	pub fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}

	pub fn paint(&self, ctx: &mut G) {
		if !self.redraw.get() {
			return;
		}
		self.redraw.set(false);
		let focus = self.focus.get();
		for (i, w) in self.widgets.iter().enumerate() {
			w.paint(ctx, focus == i);
		}
	}

	pub fn event(&self, event: Event<N>) {
		for (i, w) in self.widgets.iter().enumerate() {
			if w.event(event, self.focus.get() == i, &self.redraw) {
				if self.focus.get() != i {
					self.focus.set(i);
					self.redraw.set(true);
				}
			}
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum Event<N: Signed> {
	Init,
	Mouse {
		point: Point<N>,
		left_button: bool,
		middle_button: bool,
		right_button: bool,
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

pub trait Widget<N: Signed, C: Copy, G: Graphics<N, C>>: Any {
	fn bounds(&self) -> &Cell<Rect<N>>;
	fn paint(&self, ctx: &mut G, focused: bool);
	fn event(&self, event: Event<N>, focused: bool, redraw: &Cell<bool>) -> bool;
}

pub struct Label<N: Signed, C: Copy> {
	pub rect: Cell<Rect<N>>,
	pub color: C,
	pub label: RefCell<String>,
}

impl<N, C, G> Widget<N, C, G> for Label<N, C>
	where N: Signed, C: Copy + 'static, G: Graphics<N, C>
{
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}

	fn paint(&self, ctx: &mut G, _focused: bool) {
		let rect = self.rect.get();
		let text = self.label.borrow();
		ctx.text_center(rect, self.color, &text);
	}

	fn event(&self, _event: Event<N>, focused: bool, _redraw: &Cell<bool>) -> bool {
		focused
	}
}

pub struct Button<N: Signed, C: Copy> {
	pub rect: Cell<Rect<N>>,
	pub fg: (C, C),
	pub bg: (C, C),
	pub border: Option<(C, C)>,
	pub label: RefCell<String>,
	callback: RefCell<Rc<Fn(&Button<N, C>)>>,
	pressed: Cell<bool>,
}

impl<N: Signed> Button<N, u32> {
	pub fn new<F: Fn(&Self) + 'static>(label: String, callback: F) -> Self {
		Self {
			rect: Cell::new(Rect::default()),
			bg: (BUTTON_BACKGROUND, BUTTON_BG_SELECTION),
			fg: (BUTTON_FOREGROUND, BUTTON_FG_SELECTION),
			border: Some((BUTTON_BORDER, BUTTON_BORDER)),
			label: RefCell::new(label),
			callback: RefCell::new(Rc::new(callback)),
			pressed: Cell::new(false),
		}
	}
}

impl<N, C, G> Widget<N, C, G> for Button<N, C>
	where N: Signed, C: Copy + 'static, G: Graphics<N, C>
{
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}

	fn paint(&self, ctx: &mut G, _focused: bool) {
		let rect = self.rect.get();
		let (fg, bg, border) = if self.pressed.get() {
			(self.fg.1, self.bg.1, self.border.map(|b| b.1))
		} else {
			(self.fg.0, self.bg.0, self.border.map(|b| b.0))
		};
		let text = self.label.borrow();
		ctx.render_frame(rect, bg, border);
		ctx.text_center(rect, fg, &text);
	}

	fn event(&self, event: Event<N>, focused: bool, redraw: &Cell<bool>) -> bool {
		match event {
			Event::Mouse { point, left_button, .. } => {
				let mut click = false;
				let rect = self.rect.get();
				if rect.contains(point) {
					if left_button {
						if self.pressed.check_set(true) {
							redraw.set(true);
						}
					} else {
						if self.pressed.check_set(false) {
							click = true;
							redraw.set(true);
						}
					}
				} else if !left_button && self.pressed.check_set(false) {
					redraw.set(true);
				}
				if click {
					let cb = self.callback.borrow();
					cb(self)
				}
			}
			_ => (),
		}
		focused
	}
}