#![allow(dead_code)]
use math::*;
use specs::*;

use math::d8::*;
use renderer::*;

pub struct Frame {
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32,
}

#[derive(Default)]
pub struct Local(pub Affine<f32>);

#[derive(Default)]
pub struct Global(pub Affine<f32>);

impl Component for Global {
	type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Component for Local {
	type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

pub type TransformSystem = ::tsys::System<Transform>;

pub struct Transform;
impl ::tsys::Transform for Transform {
	type Local = Local;
	type Global = Global;
	#[inline]
	fn convert(l: &Local) -> Global {
		Global(l.0)
	}
	#[inline]
	fn combine(a: &Global, b: &Global) -> Global {
		Global(a.0 * b.0)
	}
	#[inline]
	fn rewrite(dst: &mut Global, src: &Global) {
		dst.0 = src.0
	}
}

pub struct Sprite {
	pub texture: Texture,
	pub anchor: Vector2<f32>,
	pub size: Vector2<f32>,
	pub color: [u8; 4],
	pub uv: [[u16; 2]; 4],
	pub pos: [Vector2<f32>; 4],
}

impl Sprite {
	pub fn new(texture: Texture) -> Self {
		Self {
			texture,
			anchor: Vector2::new(0.5, 0.5),
			size: Vector2::zero(),
			color: [0xff; 4],
			uv: zero_uv(),
			pos: [Vector2::new(0.0, 0.0); 4],
		}
	}
}

impl Component for Sprite {
	type Storage = VecStorage<Self>;
}

pub struct SpriteSystem;

impl<'a> System<'a> for SpriteSystem {
	type SystemData = (
		ReadStorage<'a, Global>,
		WriteStorage<'a, Sprite>,
	);
	fn run(&mut self, (tr, mut sprites): Self::SystemData) {
		//use rayon::prelude::*;
		((&tr).open().1, &mut sprites).join().for_each(|(t, s)| s.recalc_pos(t))
		//(&tr, &mut sprites).par_join().for_each(|(t, s)| s.recalc_pos(t))
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
	pub fn recalc_pos(&mut self, aff: &Global) {
		let w1 = -self.anchor.x * self.size.x;
		let w0 = w1 + self.size.x;

		let h1 = -self.anchor.y * self.size.y;
		let h0 = h1 + self.size.y;

		self.pos[0] = (aff.0.m * Vector2::new(w1, h1) + aff.0.t).into();
		self.pos[1] = (aff.0.m * Vector2::new(w0, h1) + aff.0.t).into();
		self.pos[2] = (aff.0.m * Vector2::new(w0, h0) + aff.0.t).into();
		self.pos[3] = (aff.0.m * Vector2::new(w1, h0) + aff.0.t).into();
	}
}
