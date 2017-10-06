use common::*;
use ui::*;

#[derive(Clone, Copy)]
pub struct Grid {
	pub show: bool,
	pub size: Vector<i16>,
	pub offset: Vector<i16>,
}

impl Grid {
	pub fn draw(&self, render: &mut Render, pos: Point<i16>, size: Point<i16>, zoom: i16) {
		let rr = GRID_COLOR.to_be();
		let gg = CORNER_COLOR.to_be();

		if !self.show {
			return;
		}

		let (ox, oy) = (pos.x, pos.y);

		render.ctx.set_clip_rect(Some(rect!(ox, oy, size.x * zoom, size.y * zoom)));

		let (x1, x2) = (ox, ox + size.x * zoom);
		let (y1, y2) = (oy, oy + size.y * zoom);

		let ex = size.x / self.size.x;
		let ey = size.y / self.size.y;

		let ox = ox + (self.offset.x % self.size.x) * zoom;
		let oy = oy + (self.offset.y % self.size.y) * zoom;

		let zx = self.size.x * zoom;
		let zy = self.size.y * zoom;

		for x in 0..ex + 1 {
			let x = ox + x * zx;
			render.ctx.vline(x - 1, y1, y2, rr).unwrap();
		}

		for y in 0..ey + 1 {
			let y = oy + y * zy;
			render.ctx.hline(x1, x2, y - 1, rr).unwrap();
		}

		render.ctx.set_clip_rect(None);

		// canvas border
		render.ctx.hline(x1-1, x2, y1-1, gg).unwrap();
		render.ctx.hline(x1-1, x2, y2+0, gg).unwrap();
		render.ctx.vline(x1-1, y1-1, y2, gg).unwrap();
		render.ctx.vline(x2+0, y1-1, y2, gg).unwrap();
	}
}