#![allow(dead_code)]
use cgmath::{Matrix2, Vector2, One, Zero};
use specs::{Component, VecStorage};

use d8::*;
use texture::*;


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

impl Vertex {
	#[inline]
	pub fn pack_uv(&mut self, x: f32, y: f32) {
		self.uv[0] = (x * 65535.0) as u16;
		self.uv[1] = (y * 65535.0) as u16;
	}
}

#[derive(Derivative)]
#[derivative(Default(new="true"))]
pub struct Sprite {
	#[derivative(Default(value="Matrix2::one()"))]
	pub m: Matrix2<f32>,
	#[derivative(Default(value="Vector2::zero()"))]
	pub t: Vector2<f32>,
	#[derivative(Default(value="Vector2::new(0.5, 0.5)"))]
	pub anchor: Vector2<f32>,

	pub w: f32,
	pub h: f32,

	pub color: [u8; 4],

	#[derivative(Default(value="[Vector2::new(0.0, 0.0); 4]"))]
	pub uv_cache: [Vector2<f32>; 4],
	#[derivative(Default(value="[Vector2::new(0.0, 0.0); 4]"))]
	pub pos_cache: [Vector2<f32>; 4],

	pub cache: [Vertex; 4],

	#[derivative(Default(value="None"))]
	pub texture: Option<BaseTexture>,
}

impl Component for Sprite {
	type Storage = VecStorage<Self>;
}

impl Sprite {
	#[inline]
	pub fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
		self.color = [r, g, b, a];
		for i in 0..4 {
			self.cache[i].color = [r, g, b, a];
		}
	}

	#[inline]
	pub fn set_texture(&mut self, tex: u32) {
		for i in 0..4 {
			self.cache[i].texture = tex;
		}
	}

	#[inline(always)]
	pub fn uv_n(&mut self, i: usize, u: f32, v: f32) {
		self.cache[i].pack_uv(u, v);
		self.uv_cache[i] = Vector2::new(u, v);
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

	#[inline]
	pub fn recalc(&mut self) {
		let w1 = -self.anchor.x * self.w;
		let w0 = w1 + self.w;

		let h1 = -self.anchor.y * self.h;
		let h0 = h1 + self.h;

		self.cache[0].position = (self.m * Vector2::new(w1, h1) + self.t).into();
		self.cache[1].position = (self.m * Vector2::new(w0, h1) + self.t).into();
		self.cache[2].position = (self.m * Vector2::new(w0, h0) + self.t).into();
		self.cache[3].position = (self.m * Vector2::new(w1, h0) + self.t).into();
	}
}
