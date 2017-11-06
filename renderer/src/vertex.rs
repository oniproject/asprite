#[derive(Derivative, Clone, Copy)]
#[derivative(Default)]
pub struct Vertex {
	// 4*2 + 2*2 + 4*1 + 4 = 20
	// 20 * 4 = 80 bytes per sprite instead 128
	#[derivative(Default(value="[0.0; 2]"))]
	pub position: [f32; 2],
	#[derivative(Default(value="[0; 2]"))]
	pub uv: [u16; 2],
	#[derivative(Default(value="[0xFF; 4]"))]
	pub color: [u8; 4],
	#[derivative(Default(value="0"))]
	pub texture: u32,
}

impl_vertex!(Vertex, position, uv, color, texture);

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
