use rand;
use std::num::Wrapping;

use super::*;

fn pos_from_idx(index: i32, total: i32, bounds: i32, factor: i32) -> i32 {
	let rnd = rand::random::<i32>();
	let pos = Wrapping(index) * Wrapping(bounds)
		+ (Wrapping(total * (rnd % factor)) >> 6)
		- (Wrapping(total * factor) >> 7);
	if pos.0 < 0 {
		0
	} else {
		pos.0
	}
}

fn pos_to_color(bounds: i32, pos: i32) -> i32 {
	if pos < 0 {
		0
	} else if pos >= bounds {
		bounds - 1
	} else {
		pos
	}
}

pub type GradFn = fn(index: i32, x: i16, y: i16, total: i32, bounds: i32, factor: i32) -> i32;

pub fn _basic(index: i32, _x: i16, _y: i16, total: i32, bounds: i32, factor: i32) -> i32 {
	let mut position = pos_from_idx(index, total, bounds, factor);
	position /= total;
	pos_to_color(bounds, position)
}

pub fn _dithered(index: i32, x: i16, y: i16, total: i32, bounds: i32, factor: i32) -> i32 {
	let mut pos = pos_from_idx(index, total, bounds, factor);
	let segment = ((pos << 2) / total) & 3;
	pos /= total;
	pos_to_color(bounds, match segment {
		0 if (x + y) & 1 == 0 => pos - 1,
		3 if (x + y) & 1 != 0 => pos + 1,
		_ => pos,
	})
}

pub fn extra_dithered(index: i32, x: i16, y: i16, total: i32, bounds: i32, factor: i32) -> i32 {
	let mut pos = pos_from_idx(index, total, bounds, factor);
	let segment = ((pos << 3) / total) & 7;
	pos /= total;
	pos_to_color(bounds, match segment {
		0     if (x + y) & 1 == 0         => pos - 1,
		1 | 2 if x & 1 == 0 && y & 1 == 0 => pos - 1,
		5 | 6 if x & 1 == 0 && y & 1 != 0 => pos + 1,
		7     if (x + y) & 1 != 0         => pos + 1,
		_ => pos, 
	})
}

pub fn draw_gradient<F: FnMut(Point2<i32>, i32, i32)>(r: Rect<i32>, va: Point2<i32>, vb: Point2<i32>, mut f: F) {
	if vb.x == va.x {
		if vb.y == va.y {
			return;
		}
		let total = (vb.y - va.y).abs();
		fill_rect(r, |p| {
			let idx = (vb.y - p.y as i32).abs();
			f(p, idx, total);
		});
	} else {
		let dx = (vb.x - va.x) as f64;
		let dy = (vb.y - va.y) as f64;
		let total = (dx.powf(2.0) + dy.powf(2.0)).sqrt() as i32;
		let a = dy / dx;
		let b = va.y as f64 - a * va.x as f64;
		fill_rect(r, |p| {
			let idx = {
				let (x, y) = (p.x as f64, p.y as f64);
				let (vx, vy) = (va.x as f64, va.y as f64);
				let dx = (y - vy).powf(2.0) + (x - vx).powf(2.0);
				let dy = (-a * x + y - b).powf(2.0) / (a * a + 1.0);
				(dx - dy).sqrt() as i32
			};
			f(p, idx, total);
		});
	}
}