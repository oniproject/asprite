
/*
		self.tools.paint(render);
		self.ui(render);
		self.update = render.finish();
*/

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::*;
use super::check_set::*;

pub struct Im<N: SignedInt, C: Copy + 'static> {
	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Point<N>>,
	pub callback: RefCell<Rc<Fn(&Button<N, C>)>>,
}

impl<N, C> Widget<N, C> for Button<N, C>
	where N: SignedInt, C: Copy + 'static
{
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}
	fn measured_size(&self) -> &Cell<Point<N>> {
		&self.measured
	}
	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		let rect = self.bounds().get();
		self.measured.set(Point::new(rect.dx(), rect.dy()))
	}
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, _focused: bool) {
		let rect = self.rect.get().translate(origin);
	}
	fn event(&self, event: Event<N>, origin: Point<N>, focused: bool, redraw: &Cell<bool>) -> bool {
		match event {
			_ => (),
		}
		focused
	}
}