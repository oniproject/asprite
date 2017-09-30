use rand;
use std::num::Wrapping;

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

pub fn basic(index: i32, x: i16, y: i16, total: i32, bounds: i32, factor: i32) -> i32 {
	let mut position = pos_from_idx(index, total, bounds, factor);
	position /= total;
	pos_to_color(bounds, position)
}

pub fn dithered(index: i32, x: i16, y: i16, total: i32, bounds: i32, factor: i32) -> i32 {
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