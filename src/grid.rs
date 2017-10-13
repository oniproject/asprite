use common::*;
use gui::*;

#[derive(Clone, Copy)]
pub struct Grid {
	pub show: bool,
	pub size: Vector<i16>,
	pub offset: Vector<i16>,
	pub zoom: i16,
}

impl Grid {
	pub fn paint<R: Immediate>(&self, ui: &mut R, pos: Point<i32>, size: Point<i32>) {
		if !self.show {
			return;
		}

		let pos = Point::new(pos.x as i16, pos.y as i16);
		let size = Point::new(size.x as i16, size.y as i16);
		let zoom = self.zoom;

		let rr = GRID_COLOR;
		let gg = CORNER_COLOR;

		let (ox, oy) = (pos.x, pos.y);

		let (x1, x2) = (ox, ox + size.x * zoom);
		let (y1, y2) = (oy, oy + size.y * zoom);

		let ex = size.x / self.size.x;
		let ey = size.y / self.size.y;

		let ox = ox + (self.offset.x % self.size.x) * zoom;
		let oy = oy + (self.offset.y % self.size.y) * zoom;

		let zx = self.size.x * zoom;
		let zy = self.size.y * zoom;

		ui.clip(Some(Rect::with_size(pos.x, pos.y, size.x * zoom, size.y * zoom)));
		for x in 0..ex + 1 {
			let x = ox + x * zx;
			ui.vline(x - 1, y1, y2, rr);
		}
		for y in 0..ey + 1 {
			let y = oy + y * zy;
			ui.hline(x1, x2, y - 1, rr);
		}
		ui.clip(None);

		// canvas border
		ui.hline(x1-1, x2, y1-1, gg);
		ui.hline(x1-1, x2, y2+0, gg);
		ui.vline(x1-1, y1-1, y2, gg);
		ui.vline(x2+0, y1-1, y2, gg);
	}
}