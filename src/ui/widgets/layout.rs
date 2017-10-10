use common::*;
use super::*;

use std::rc::Rc;

pub fn layout_vertical<N: Signed, W: Bounds<N> + ?Sized>(rect: Rect<N>, widgets: &Vec<Rc<W>>) {
	let width = rect.dx();
	let mut min = rect.min;
	for w in widgets.iter() {
		let r = {
			let r = w.bounds().get();
			let w = width;
			let h = r.dy();
			let x = min.x;
			let y = min.y;
			min.y += h;
			Rect::with_size(x, y, w, h)
		};
		w.bounds().set(r);
	}
}

pub fn layout_horizontal<N: Signed, W: Bounds<N> + ?Sized>(rect: Rect<N>, widgets: &Vec<Rc<W>>) {
	let height = rect.dy();
	let mut min = rect.min;
	for w in widgets.iter() {
		let r = {
			let r = w.bounds().get();
			let w = r.dx();
			let h = height;
			let x = min.x;
			let y = min.y;
			min.y += h;
			Rect::with_size(x, y, w, h)
		};
		w.bounds().set(r);
	}
}