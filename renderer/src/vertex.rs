
#[inline(always)]
pub fn pack_uv(u: f32, v: f32) -> [u16; 2] {
	let u = (u * 65535.0) as u16;
	let v = (v * 65535.0) as u16;
	[u, v]
}

#[inline(always)]
pub const fn zero_uv() -> [[u16; 2]; 4] {
	[
		[0x0000, 0x0000],
		[0xFFFF, 0x0000],
		[0xFFFF, 0xFFFF],
		[0x0000, 0xFFFF],
	]
}
