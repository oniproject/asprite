use std::cell::Cell;
use std::rc::Rc;

use common::*;
use super::*;

/*
pub struct Container<N: Signed, C: Copy + 'static, G: Graphics<N, C>> {
	pub widgets: Vec<Rc<Widget<N, C, G>>>,
	pub rect: Cell<Rect<N>>,
}

impl<N: Signed, C: Copy + 'static, G: Graphics<N, C>> Bounds<N> for Container<N, C, G> {
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}
}

impl<N, C, Canvas> Container<N, C, G>
	where N: Signed, C: Copy + 'static
{
	fn paint(&self, ctx: &mut Graphics<N, C, Canvas=Canvas>, _focused: bool) {
		/*
		let rect = self.rect.get();
		let text = self.label.borrow();
		ctx.text_center(rect, self.color, &text);
		*/
	}

	fn event(&self, _event: Event<N>, focused: bool, _redraw: &Cell<bool>) -> bool {
		focused
	}
}
*/

pub struct Window<N: Signed, C: Copy + 'static, G: Graphics<N, C>> {
	pub widgets: Vec<Rc<Widget<N, C, G>>>,
	pub focus: Cell<usize>,
	pub rect: Cell<Rect<N>>,
	pub redraw: Cell<bool>,
	pub mouse_pos: Cell<Point<N>>,
	pub mouse_left: Cell<bool>,
	pub bg: C,
}

impl<N: Signed, C: Copy + 'static, G: Graphics<N, C>> Window<N, C, G> {
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
		ctx.render_frame(self.rect.get(), self.bg, None);

		let focus = self.focus.get();
		for (i, w) in self.widgets.iter().enumerate() {
			w.paint(ctx, focus == i);
		}
	}

	pub fn event(&self, event: Mouse<N>) {
		match event {
			Mouse::Move(p) => self.mouse_pos.set(p),
			Mouse::Press(p) => {
				self.mouse_pos.set(p);
				self.mouse_left.set(true);
			}
			Mouse::Release(p) => {
				self.mouse_pos.set(p);
				self.mouse_left.set(false);
			}
		}
		let event = Event::Mouse {
			point: self.mouse_pos.get(),
			left: self.mouse_left.get(),
			right: false,
			middle: false,
		};
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
