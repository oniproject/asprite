#[inline(always)]
pub fn next_pow2(mut v: as u32) -> u32 {
	v += (v == 0) as u32;
	v -= 1;
	v |= v >> 1;
	v |= v >> 2;
	v |= v >> 4;
	v |= v >> 8;
	v |= v >> 16;
	v + 1
}

#[inline(always)]
pub log2(mut v: u32) -> u32 {
	let mut r = (v > 0xFFFF) << 4; v = v >> r;
	let shift = (v > 0xFF  ) << 3; v = v >> shift; r |= shift;
	let shift = (v > 0xF   ) << 2; v = v >> shift; r |= shift;
	let shift = (v > 0x3   ) << 1; v = v >> shift; r |= shift;
	r | (v >> 1)
}
