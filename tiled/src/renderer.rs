use na::Point2;
use std::mem::swap;

enum DrawOrder {
    RightUp,
	LeftDown,
	LeftUp,
    RightDown,
}

struct Rect {
	min: Point2<isize>,
	max: Point2<isize>,
}

impl DrawOrder {
	pub fn run<F>(self, r: Rect, f: F)
		where F: Fn(isize, isize)
	{
		let Rect { mut min, mut max } = r;
		let (ix, iy) = match self {
			DrawOrder::RightDown => {
				(1, 1)
			}
			DrawOrder::RightUp => {
				swap(&mut min.y, &mut max.y);
				(1, -1)
			}
			DrawOrder::LeftDown => {
				swap(&mut min.x, &mut max.x);
				(-1, 1)
			}
			DrawOrder::LeftUp => {
				swap(&mut min.x, &mut max.x);
				swap(&mut min.y, &mut max.y);
				(-1, -1)
			}
		};

		max.x += ix;
		max.y += iy;

		let mut y = min.x;
		while y != max.y {
			let mut x = min.x;
			while x != max.x {
				f(x, y);
				x += ix;
			}
			y += iy;
		}
	}
}

pub trait Painter {
	fn line(&mut self, x1: isize, x1: isize, x2: isize, y2: isize);
	fn set_dash_offset(&mut self, offset: isize);
	fn set_grid_pen(&mut self);
}

pub trait CoordSystem<P> {
	fn pixel2tile(&self, P) -> P;
	fn tile2pixel(&self, P) -> P;
	fn screen2tile(&self, P) -> P;
	fn tile2screen(&self, P) -> P;
	fn screen2pixel(&self, P) -> P;
	fn pixel2screen(&self, P) -> P;

	fn grid(&self, Pt, Pt, Option<Pti>, &mut Painter);
	fn selection(&self);
	fn tiled(&self);
}

pub type Pt = Point2<f32>;
pub type Pti = Point2<isize>;

pub struct Orthogonal {
	pub tile_width: f32,
	pub tile_height: f32,
}

impl CoordSystem<Pt> for Orthogonal {
	fn pixel2tile(&self, p: Pt) -> Pt {
		Pt::new(p.x / self.tile_width, p.y / self.tile_height)
	}
	fn tile2pixel(&self, p: Pt) -> Pt {
		Pt::new(p.x * self.tile_width, p.y * self.tile_height)
	}
	fn screen2tile(&self, p: Pt) -> Pt {
		Pt::new(p.x / self.tile_width, p.y / self.tile_height)
	}
	fn tile2screen(&self, p: Pt) -> Pt {
		Pt::new(p.x * self.tile_width, p.y * self.tile_height)
	}
	fn screen2pixel(&self, p: Pt) -> Pt {
		p
	}
	fn pixel2screen(&self, p: Pt) -> Pt {
		p
	}

	fn grid(&self, min: Pt, max: Pt, size: Option<Pti>, p: &mut Painter) {
		let tw = self.tile_width;
		let th = self.tile_height;
		assert!(tw > 0.0 && th > 0.0);

		let mut min_x = ((min.x / tw).floor() * tw) as isize;
		let mut min_y = ((min.y / th).floor() * th) as isize;
		let mut max_x = (max.x.ceil()) as isize;
		let mut max_y = (max.y.ceil()) as isize;

		let (tw, th) = (tw as isize, th as isize);
		if let Some(size) = size {
			min_x = min_x.max(0);
			min_y = min_y.max(0);
			max_x = max_x.min(size.x * tw + 1);
			max_y = max_y.min(size.y * th + 1);
		}

		p.set_grid_pen();
		p.set_dash_offset(min_y);
		for x in (min_x..max_x).step_by(tw as usize) {
			p.line(x, min_y, x, max_y - 1);
		}
		p.set_dash_offset(min_x);
		for y in (min_y..max_y).step_by(th as usize) {
			p.line(min_x, y, max_x - 1, y);
		}
	}

	fn selection(&self) {
		unimplemented!();
		/*
		fn selection(QPainter *painter,
												const QRegion &region,
												const QColor &color,
												const QRectF &exposed) const
		{
		foreach (const QRect &r, region.rects()) {
			const QRectF toFill = QRectF(boundingRect(r)).intersected(exposed);
			if (!toFill.isEmpty())
				painter->fillRect(toFill, color);
		}
		*/
	}

	fn tiled(&self) {
		unimplemented!();
	}
}