use std::cell::{Cell, RefCell};
use super::*;

pub struct Label<N: Num, C: Copy> {
	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Point<N>>,

	pub color: C,
	pub label: RefCell<String>,
}

impl<N, C> Widget<N, C> for Label<N, C>
	where N: Num, C: Copy + 'static
{
	fn bounds(&self) -> &Cell<Rect<N>> { &self.rect }
	fn measured_size(&self) -> &Cell<Point<N>> { &self.measured }

	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		let rect = self.bounds().get();
		self.measured.set(Point::new(rect.dx(), rect.dy()))
	}
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, _focused: bool) {
		let rect = self.rect.get().translate(origin);
		let text = self.label.borrow();
		ctx.text_center(rect, self.color, &text);
	}
}