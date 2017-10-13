use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::*;
use super::check_set::*;

pub struct Button<N: SignedInt, C: Copy + 'static> {
	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Point<N>>,
	pub layout: RefCell<FlowData<N>>,

	pub fg: (C, C),
	pub bg: (C, C),
	pub border: Option<(C, C)>,
	pub label: RefCell<String>,
	pub callback: RefCell<Rc<Fn(&Button<N, C>)>>,
	pub pressed: Cell<bool>,
}

impl<N, C> Widget<N, C> for Button<N, C>
	where N: SignedInt, C: Copy + 'static
{
	fn bounds(&self) -> &Cell<Rect<N>> { &self.rect }
	fn layout_data(&self) -> Option<Ref<Any>> { Some(self.layout.borrow()) }
	fn measured_size(&self) -> &Cell<Point<N>> { &self.measured }

	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		let rect = self.bounds().get();
		self.measured.set(Point::new(rect.dx(), rect.dy()))
	}

	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, _focused: bool) {
		let rect = self.rect.get().translate(origin);
		let (fg, bg, border) = if self.pressed.get() {
			(self.fg.1, self.bg.1, self.border.map(|b| b.1))
		} else {
			(self.fg.0, self.bg.0, self.border.map(|b| b.0))
		};
		let text = self.label.borrow();
		ctx.fill(rect, bg);
		if let Some(border) = border {
			ctx.border(rect, border);
		}
		ctx.text_center(rect, fg, &text);
	}

	fn event(&self, event: Event<N>, origin: Point<N>, focused: bool, redraw: &Cell<bool>) -> bool {
		match event {
			Event::Mouse { point, left, .. } => {
				let mut click = false;
				let rect = self.rect.get().translate(origin);
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
				} else if self.pressed.check_set(false) {
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