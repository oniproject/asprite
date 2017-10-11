use std::cell::{Cell, RefCell};
use std::rc::Rc;

use common::*;
use super::*;
use super::check_set::*;

pub struct Button<N: SignedInt, C: Copy + 'static> {
	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Point<N>>,
	pub fg: (C, C),
	pub bg: (C, C),
	pub border: Option<(C, C)>,
	pub label: RefCell<String>,
	pub callback: RefCell<Rc<Fn(&Button<N, C>)>>,
	pub pressed: Cell<bool>,
	pub flow: FlowData<N>,
}

impl<N: SignedInt, C: Copy + 'static> Bounds<N> for Button<N, C> {
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}
}

impl<N: SignedInt, C: Copy + 'static> Layout for Button<N, C> {
	fn layout_data(&self) -> Option<&Any> {
		Some(&self.flow)
	}
}

impl<N: SignedInt, C: Copy + 'static> Measured<N> for Button<N, C> {
	fn measured_size(&self) -> &Cell<Point<N>> {
		&self.measured
	}
	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		let rect = self.bounds().get();
		self.measured.set(Point::new(rect.dx(), rect.dy()))
	}
}

impl<N, C> Widget<N, C> for Button<N, C>
	where N: SignedInt, C: Copy + 'static
{
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, _focused: bool) {
		let rect = self.rect.get().translate(origin);
		let (fg, bg, border) = if self.pressed.get() {
			(self.fg.1, self.bg.1, self.border.map(|b| b.1))
		} else {
			(self.fg.0, self.bg.0, self.border.map(|b| b.0))
		};
		let text = self.label.borrow();
		ctx.render_rect(rect, bg);
		if let Some(border) = border {
			ctx.render_border(rect, border);
		}
		ctx.render_text_center(rect, fg, &text);
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