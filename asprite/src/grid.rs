use math::*;
use gui::*;

#[derive(Clone, Copy)]
pub struct Grid {
	pub show: bool,
	pub rect: Rect<i32>,
	pub size: Vector2<i16>,
	pub offset: Vector2<i16>,
	pub zoom: i16,
}

impl Grid {
	pub fn paint<R: Graphics<i16, u32>>(&self, ctx: &mut R) {
		if !self.show {
			return;
		}

		let (pos, size) = {
			let r = self.rect;
			let min = r.min;
			let pos = Point2::new(min.x as i16, min.y as i16);
			let size = Point2::new(r.dx() as i16, r.dy() as i16);
			(pos, size)
		};

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

		ctx.clip(Some(Rect::with_size(pos.x, pos.y, size.x * zoom, size.y * zoom)));
		for x in 0..ex + 1 {
			let x = ox + x * zx;
			ctx.vline(x - 1, y1, y2, rr);
		}
		for y in 0..ey + 1 {
			let y = oy + y * zy;
			ctx.hline(x1, x2, y - 1, rr);
		}
		ctx.clip(None);

		// canvas border
		ctx.hline(x1-1, x2, y1-1, gg);
		ctx.hline(x1-1, x2, y2+0, gg);
		ctx.vline(x1-1, y1-1, y2, gg);
		ctx.vline(x2+0, y1-1, y2, gg);
	}
}