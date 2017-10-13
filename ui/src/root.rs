use std::cell::{Ref, RefMut, RefCell, Cell};
use std::rc::Rc;

use super::*;

pub struct Root<N: Num, C: Copy + 'static> {
	pub widgets: RefCell<Vec<Rc<Widget<N, C>>>>,
	pub focus: Cell<usize>,
	pub rect: Cell<Rect<N>>,
	pub redraw: Cell<bool>,
	pub mouse_pos: Cell<Point<N>>,
	pub mouse_left: Cell<bool>,
	pub bg: Cell<C>,
}

impl<N: Num, C: Copy + 'static> Container for Root<N, C> {
	type Storage = Vec<Rc<Widget<N, C>>>;
	type Item = Rc<Widget<N, C>>;

	fn children(&self) -> Ref<Self::Storage> {
		self.widgets.borrow()
	}
	fn children_mut(&self) -> RefMut<Self::Storage> {
		self.widgets.borrow_mut()
	}
	fn add(&self, w: Self::Item) {
		let mut widgets = self.children_mut();
		widgets.push(w);
	}
	fn insert(&self, index: usize, w: Self::Item) {
		let mut widgets = self.widgets.borrow_mut();
		widgets.insert(index, w);
	}
	fn remove(&self, index: usize) -> Self::Item {
		let mut widgets = self.widgets.borrow_mut();
		widgets.remove(index)
	}
}

impl<N: Num, C: Copy + 'static> Root<N, C> {
	pub fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}

	pub fn measure(&self) {
		let widgets = self.widgets.borrow();
		for c in widgets.iter() {
			c.measure(None, None);
		}
	}

	pub fn layout(&self) {
		let widgets = self.widgets.borrow();
		for c in widgets.iter() {
			c.layout();
		}
	}

	pub fn paint(&self, ctx: &mut Graphics<N, C>) {
		if !self.redraw.get() {
			return;
		}
		self.redraw.set(false);
		ctx.fill(self.rect.get(), self.bg.get());

		let origin = self.rect.get().min;
		let focus = self.focus.get();
		let widgets = self.widgets.borrow();
		for (i, w) in widgets.iter().enumerate() {
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
		let widgets = self.widgets.borrow();
		for (i, w) in widgets.iter().enumerate() {
			if w.event(event, origin, self.focus.get() == i, &self.redraw) {
				if self.focus.get() != i {
					self.focus.set(i);
					self.redraw.set(true);
				}
			}
		}
	}
}
