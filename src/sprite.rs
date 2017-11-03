#![allow(dead_code)]
use cgmath::{Vector2, Zero};
use specs::{self, Component, VecStorage, Join};

use d8::*;
use texture::*;
use transform::*;

pub struct Frame {
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32,
}

#[derive(Derivative, Clone, Copy)]
#[derivative(Default(new="true"))]
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

pub struct Sprite {
	pub texture: BaseTexture,
	pub anchor: Vector2<f32>,
	pub size: Vector2<f32>,
	pub color: [u8; 4],
	pub uv: [[u16; 2]; 4],
	pub pos: [Vector2<f32>; 4],
}

impl Sprite {
	pub fn new(texture: BaseTexture) -> Self {
		Self {
			texture,
			anchor: Vector2::new(0.5, 0.5),
			size: Vector2::zero(),
			color: [0xff; 4],
			uv: [
				[0x0000, 0x0000],
				[0xFFFF, 0x0000],
				[0xFFFF, 0xFFFF],
				[0x0000, 0xFFFF],
			],
			pos: [Vector2::new(0.0, 0.0); 4],
		}
	}
}

impl Component for Sprite {
	type Storage = VecStorage<Self>;
}

pub struct SpriteSystem;

impl<'a> specs::System<'a> for SpriteSystem {
	type SystemData = (
		specs::ReadStorage<'a, Affine<f32>>,
		specs::WriteStorage<'a, Sprite>,
	);
	fn run(&mut self, (tr, mut sprites): Self::SystemData) {
		((&tr).open().1, &mut sprites).join()
			.for_each(|(t, s)| s.recalc_pos(t))
	}
}

impl Sprite {
	#[inline(always)]
	pub fn uv_n(&mut self, i: usize, u: f32, v: f32) {
		self.uv[i] = pack_uv(u, v);
	}

	#[inline]
	pub fn uv(&mut self) {
		self.uv_n(0, 0.0, 0.0);
		self.uv_n(1, 1.0, 0.0);
		self.uv_n(2, 1.0, 1.0);
		self.uv_n(3, 0.0, 1.0);
	}

	#[inline]
	pub fn frame_uv(&mut self, f: Frame, tw: f32, th: f32) {
		let a = f.x / tw;
		let b = (f.x + f.w) / tw;
		let c = f.y / th;
		let d = (f.y + f.h) / th;

		self.uv_n(0, a, c);
		self.uv_n(1, b, c);
		self.uv_n(2, b, d);
		self.uv_n(3, a, d);
	}

	#[inline]
	pub fn frame_uv_rotated(&mut self, f: Frame, tw: f32, th: f32, rotate: D8) {
		// width and height div 2 div baseFrame size
		let w2 = f.w / 2.0 / tw;
		let h2 = f.h / 2.0 / th;

		// coordinates of center
		let cx = (f.x / tw) + w2;
		let cy = (f.y / th) + h2;

		let rotate = rotate.add(D8_NW); // NW is top-left corner
		self.uv_n(0,
			cx + (w2 * rotate.ux()),
			cy + (h2 * rotate.uy()),
		);
		let rotate = rotate.add(D8_S); // rotate 90 degrees clockwise
		self.uv_n(1,
			cx + (w2 * rotate.ux()),
			cy + (h2 * rotate.uy()),
		);
		let rotate = rotate.add(D8_S);
		self.uv_n(2,
			cx + (w2 * rotate.ux()),
			cy + (h2 * rotate.uy()),
		);
		let rotate = rotate.add(D8_S);
		self.uv_n(3,
			cx + (w2 * rotate.ux()),
			cy + (h2 * rotate.uy()),
		);
	}

	#[inline(always)]
	pub fn recalc_pos(&mut self, aff: &Affine<f32>) {
		let w1 = -self.anchor.x * self.size.x;
		let w0 = w1 + self.size.x;

		let h1 = -self.anchor.y * self.size.y;
		let h0 = h1 + self.size.y;

		self.pos[0] = (aff.m * Vector2::new(w1, h1) + aff.t).into();
		self.pos[1] = (aff.m * Vector2::new(w0, h1) + aff.t).into();
		self.pos[2] = (aff.m * Vector2::new(w0, h0) + aff.t).into();
		self.pos[3] = (aff.m * Vector2::new(w1, h0) + aff.t).into();
	}
}
