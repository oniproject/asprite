use std::cell::{Cell, RefCell};
use super::*;

pub struct Label<N: BaseNum, C: Copy> {
	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Vector2<N>>,

	pub color: C,
	pub label: RefCell<String>,
}

impl<N, C> Widget<N, C> for Label<N, C>
	where N: BaseNum + 'static, C: Copy + 'static
{
	fn bounds(&self) -> &Cell<Rect<N>> { &self.rect }
	fn measured_size(&self) -> &Cell<Vector2<N>> { &self.measured }

	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		let rect = self.bounds().get();
		self.measured.set(Vector2::new(rect.dx(), rect.dy()))
	}
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Vector2<N>, _focused: bool) {
		let rect = self.rect.get().translate(origin);
		let text = self.label.borrow();
		ctx.text_center(rect, self.color, &text);
	}
}