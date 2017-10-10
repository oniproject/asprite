use std::cell::{Cell, RefCell};

use common::*;
use super::*;

pub struct Label<N: Signed, C: Copy> {
	pub rect: Cell<Rect<N>>,
	pub color: C,
	pub label: RefCell<String>,
}

impl<N: Signed, C: Copy + 'static> Bounds<N> for Label<N, C> {
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}
}

impl<'a, N, C, G> Widget<N, C, G> for Label<N, C>
	where N: Signed, C: Copy + 'static, G: Graphics<N, C>
{
	fn paint(&self, ctx: &mut G, _focused: bool) {
		let rect = self.rect.get();
		let text = self.label.borrow();
		ctx.text_center(rect, self.color, &text);
	}

	fn event(&self, _event: Event<N>, focused: bool, _redraw: &Cell<bool>) -> bool {
		focused
	}
}