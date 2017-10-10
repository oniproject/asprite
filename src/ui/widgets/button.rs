use std::cell::{Cell, RefCell};
use std::rc::Rc;

use common::*;
use super::*;
use super::check_set::*;

pub struct Button<N: Signed, C: Copy + 'static> {
	pub rect: Cell<Rect<N>>,
	pub fg: (C, C),
	pub bg: (C, C),
	pub border: Option<(C, C)>,
	pub label: RefCell<String>,
	pub callback: RefCell<Rc<Fn(&Button<N, C>)>>,
	pub pressed: Cell<bool>,
}

impl<N: Signed, C: Copy + 'static> Bounds<N> for Button<N, C> {
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}
}

impl<N, C, G> Widget<N, C, G> for Button<N, C>
	where N: Signed, C: Copy + 'static, G: Graphics<N, C>
{
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
			Event::Mouse { point, left, .. } => {
				let mut click = false;
				let rect = self.rect.get();
				if rect.contains(point) {
					if left {
						if self.pressed.check_set(true) {
							redraw.set(true);
						}
					} else {
						if self.pressed.check_set(false) {
							click = true;
							redraw.set(true);
						}
					}
				} else if !left && self.pressed.check_set(false) {
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