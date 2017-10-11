use std::cell::Cell;
use std::rc::Rc;

use common::*;
use super::*;

pub struct Root<N: SignedInt, C: Copy + 'static> {
	pub widgets: Vec<Rc<Widget<N, C>>>,
	pub focus: Cell<usize>,
	pub rect: Cell<Rect<N>>,
	pub redraw: Cell<bool>,
	pub mouse_pos: Cell<Point<N>>,
	pub mouse_left: Cell<bool>,
	pub bg: C,
}

impl<N: SignedInt, C: Copy + 'static> Root<N, C> {
	pub fn add(&mut self, w: Rc<Widget<N, C>>) {
		self.widgets.push(w);
	}

	pub fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}

	pub fn measure(&self) {
		for c in &self.widgets {
			c.measure(None, None);
		}
	}

	pub fn layout(&self) {
		for c in &self.widgets {
			c.layout();
		}
	}

	pub fn paint(&self, ctx: &mut Graphics<N, C>) {
		if !self.redraw.get() {
			return;
		}
		self.redraw.set(false);
		ctx.render_rect(self.rect.get(), self.bg);

		let origin = self.rect.get().min;
		let focus = self.focus.get();
		for (i, w) in self.widgets.iter().enumerate() {
			w.paint(ctx, origin, focus == i);
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
		let origin = self.rect.get().min;
		for (i, w) in self.widgets.iter().enumerate() {
			if w.event(event, origin, self.focus.get() == i, &self.redraw) {
				if self.focus.get() != i {
					self.focus.set(i);
					self.redraw.set(true);
				}
			}
		}
	}
}
