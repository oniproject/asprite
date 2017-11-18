use std::ops::DerefMut;

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

/// Defeat borrowchecker
/// https://stackoverflow.com/questions/29570781/temporarily-move-out-of-borrowed-content
#[inline(always)]
pub fn temporarily_move_out<T, D, F>(to: D, f: F)
	where D: DerefMut<Target=T>, F: FnOnce(T) -> T
{
	use std::mem::{forget, uninitialized, replace};
	let mut to = to;
	let tmp = replace(&mut *to, unsafe { uninitialized() });
	let new = f(tmp);
	let uninit = replace(&mut *to, new);
	forget(uninit);
}

