use std::cell::{Cell, RefCell};

use common::*;
use super::*;

pub struct Label<N: SignedInt, C: Copy> {
	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Point<N>>,
	pub color: C,
	pub label: RefCell<String>,
}

impl<N: SignedInt, C: Copy + 'static> Bounds<N> for Label<N, C> {
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}
}

impl<N: SignedInt, C: Copy + 'static> Layout for Label<N, C> {}

impl<N: SignedInt, C: Copy + 'static> Measured<N> for Label<N, C> {
	fn measured_size(&self) -> &Cell<Point<N>> {
		&self.measured
	}
	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		let rect = self.bounds().get();
		self.measured.set(Point::new(rect.dx(), rect.dy()))
	}
}

impl<N, C> Widget<N, C> for Label<N, C>
	where N: SignedInt, C: Copy + 'static
{
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, _focused: bool) {
		let rect = self.rect.get().translate(origin);
		let text = self.label.borrow();
		ctx.render_text_center(rect, self.color, &text);
	}
	fn event(&self, _event: Event<N>, _origin: Point<N>, focused: bool, _redraw: &Cell<bool>) -> bool {
		focused
	}
}