use vulkano::command_buffer::AutoCommandBufferBuilder;
use std::cell::{Cell, RefCell};

use ui;
use math::*;
use renderer::*;

enum Command {
	Quad([u8; 4], Rect<f32>),
	Texture(Texture, Rect<f32>),
	TextureFrame(Texture, Rect<f32>, Rect<f32>),
	Text(Point2<f32>, [u8; 4], String),
}

pub struct Graphics<'a> {
	buffer: RefCell<Vec<Command>>,
	pub font: Font<'a>,
	font_size: f32,
	hovered: Cell<bool>,
}

#[inline]
pub fn gen_frame_uv(r: Rect<f32>, tw: f32, th: f32) -> [[u16;2]; 4] {
	let a = r.min.x / tw;
	let b = r.max.x / tw;
	let c = r.min.y / th;
	let d = r.max.y / th;

	[
		pack_uv(a, c),
		pack_uv(b, c),
		pack_uv(b, d),
		pack_uv(a, d),
	]
}

impl<'a> Graphics<'a> {
	pub fn new(font: Font<'a>, font_size: f32) -> Self {
		Self {
			buffer: RefCell::new(Vec::new()),
			font,
			font_size,
			hovered: Cell::new(false),
		}
	}
	pub fn run(&mut self, mut cb: AutoCommandBufferBuilder, renderer: &mut Renderer) -> errors::Result<AutoCommandBufferBuilder> {
		const COLOR: [u8; 4] = [0xFF; 4];
		const UV: [[u16; 2]; 4] = zero_uv();

		let mut sprite = false;
		let mut buf = self.buffer.borrow_mut();
		for cmd in buf.drain(..) {
			match cmd {
			Command::Quad(c, r) => {
				cb = if sprite { cb } else { sprite = true; renderer.start_sprites(cb)? };
				cb = renderer.color_quad(cb, r.min.to_vec(), r.max.to_vec(), c)?;
			}
			Command::Texture(t, r) => {
				let Rect { min, max } = r;
				let pos = [
					Vector2::new(min.x, min.y),
					Vector2::new(max.x, min.y),
					Vector2::new(max.x, max.y),
					Vector2::new(min.x, max.y),
				];
				cb = if sprite { cb } else { sprite = true; renderer.start_sprites(cb)? };
				cb = renderer.texture_quad(cb, t, COLOR, &pos, &UV)?;
			}
			Command::TextureFrame(t, r, f) => {
				let Rect { min, max } = r;
				let pos = [
					Vector2::new(min.x, min.y),
					Vector2::new(max.x, min.y),
					Vector2::new(max.x, max.y),
					Vector2::new(min.x, max.y),
				];
				let uv = gen_frame_uv(f, t.wh.0 as f32, t.wh.1 as f32);
				cb = if sprite { cb } else { sprite = true; renderer.start_sprites(cb)? };
				cb = renderer.texture_quad(cb, t, COLOR, &pos, &uv)?;
			}
			Command::Text(p, c, s) => {
				// TODO: reduce reallocations
				let lay: Vec<_> = renderer.text_lay(&self.font, self.font_size, &s, p.x, p.y).collect();
				cb = if !sprite { cb } else { sprite = false; renderer.end_sprites(cb)? };
				cb = renderer.glyphs(cb, &lay, c)?;
			}
			}
		}

		Ok(if !sprite { cb } else { renderer.end_sprites(cb)? })
	}
}

impl<'a> ui::Graphics for Graphics<'a> {
	type Texture = Texture;
	type Color = [u8; 4];
	#[inline]
	fn set_hovered(&self) {
		self.hovered.set(true);
	}
	#[inline]
	fn texture_dimensions(&self, texture: &Self::Texture) -> Vector2<f32> {
		let wh = texture.wh;
		Vector2::new(wh.0 as f32, wh.1 as f32)
	}
	#[inline]
	fn quad(&self, color: Self::Color, rect: &Rect<f32>) {
		self.buffer.borrow_mut().push(Command::Quad(color, *rect))
	}
	#[inline]
	fn texture(&self, texture: &Self::Texture, rect: &Rect<f32>) {
		self.buffer.borrow_mut().push(Command::Texture(texture.clone(), *rect))
	}
	#[inline]
	fn texture_frame(&self, texture: &Self::Texture, rect: &Rect<f32>, frame: &Rect<f32>) {
		self.buffer.borrow_mut().push(Command::TextureFrame(texture.clone(), *rect, *frame))
	}
	#[inline]
	fn measure_text(&self, _text: &str) -> Vector2<f32> {
		unimplemented!()
	}
	#[inline]
	fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
		self.buffer.borrow_mut().push(Command::Text(base, color, text.to_string()))
	}
}
