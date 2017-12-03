use vulkano::command_buffer::AutoCommandBufferBuilder;

use lyon::path::Path;
use lyon::tessellation::geometry_builder::{VertexConstructor, VertexBuffers, BuffersBuilder};
use lyon::tessellation::{FillTessellator, FillOptions};
use lyon::tessellation::FillVertex;
use lyon::tessellation::StrokeVertex;

use std::sync::{Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};

use ui;
use math::*;
use renderer::*;

type GpuFillVertex = vg::Vertex;

enum Command {
	Quad([u8; 4], Rect<f32>),
	Texture(Texture, Rect<f32>),
	TextureFrame(Texture, Rect<f32>, Rect<f32>),
	Text(Point2<f32>, [u8; 4], String),

	Custom(CustomCmd),
}

pub enum CustomCmd {
	Fill([u8;4], Affine<f32>, Path),
}

pub struct Graphics {
	buffer: Vec<Command>,
	pub font: Font<'static>,
	glyphs: Vec<rusttype::PositionedGlyph<'static>>,
	string_buf: String,
	font_size: f32,
	pub hovered: AtomicBool,
	vg: Mutex<VertexBuffers<GpuFillVertex>>,
}

unsafe impl Send for Graphics {}
unsafe impl Sync for Graphics {}

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

impl Graphics {
	pub fn new(font: Font<'static>, font_size: f32) -> Self {
		Self {
			buffer: Vec::new(),
			hovered: AtomicBool::new(false),
			vg: Mutex::new(VertexBuffers::new()),
			glyphs: Vec::new(),
			string_buf: String::new(),
			font,
			font_size,
		}
	}

	#[inline(always)]
	fn push(&self, cmd: Command)  {
		unsafe {
			let buf = &self.buffer as (*const Vec<Command>) as (*mut Vec<Command>);
			let buf = &mut *buf;
			buf.push(cmd);
		}
	}

	pub fn is_hovered(&self) -> bool {
		self.hovered.load(Ordering::Relaxed)
	}

	pub fn run(&mut self, mut cb: AutoCommandBufferBuilder, renderer: &mut Renderer) -> errors::Result<AutoCommandBufferBuilder> {
		self.hovered.store(false, Ordering::Relaxed);

		const COLOR: [u8; 4] = [0xFF; 4];
		const UV: [[u16; 2]; 4] = zero_uv();

		let mut sprite = false;
		let mut vg = self.vg.lock().unwrap();
		let buf = &mut self.buffer;
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
			Command::Text(start, c, text) => {
				use rusttype::{point, Scale};

				let scale = Scale::uniform(self.font_size);
				let v = self.font.v_metrics(scale);

				let start = start + Vector2::new(0.0, v.ascent + v.descent);
				let start = point(start.x.trunc(), start.y.trunc());
				let font = &self.font;
				let iter = text.chars()
					.filter_map(|c| font.glyph(c))
					.scan((None, 0.0), |save, g| {
						let g = g.scaled(scale);
						if let Some(last) = save.0 {
							save.1 += font.pair_kerning(scale, last, g.id());
						}
						let g = g.positioned(point(start.x + save.1, start.y));
						save.1 += g.unpositioned().h_metrics().advance_width;
						save.0 = Some(g.id());
						Some(g)
					});

				self.glyphs.clear();
				self.glyphs.extend(iter);

				cb = if !sprite { cb } else { sprite = false; renderer.end_sprites(cb)? };
				cb = renderer.glyphs(cb, &self.glyphs, c)?;
			}

			Command::Custom(CustomCmd::Fill(color, proj, path)) => {
				let mut tessellator = FillTessellator::new();
				{
					tessellator.tessellate_path(
						path.path_iter(),
						&FillOptions::tolerance(0.01),
						&mut BuffersBuilder::new(&mut vg, VertexCtor { proj, color }),
					).unwrap();
				}

				cb = if !sprite { cb } else { sprite = false; renderer.end_sprites(cb)? };
				cb = renderer.start_vg(cb)?;
				cb = renderer.path(cb, &vg.vertices, &vg.indices)?;
				vg.vertices.clear();
				vg.indices.clear();
				cb = renderer.end_vg(cb)?;
			}
			}
		}

		self.string_buf.clear();

		Ok(if !sprite { cb } else { renderer.end_sprites(cb)? })
	}
}

impl ui::Graphics for Graphics {
	type Texture = Texture;
	type Color = [u8; 4];
	type Custom = CustomCmd;

	#[inline]
	fn set_hovered(&self) {
		self.hovered.store(true, Ordering::Relaxed);
	}
	#[inline]
	fn texture_dimensions(&self, texture: &Self::Texture) -> Vector2<f32> {
		let wh = texture.wh;
		Vector2::new(wh.0 as f32, wh.1 as f32)
	}
	#[inline]
	fn quad(&self, color: Self::Color, rect: &Rect<f32>) {
		self.push(Command::Quad(color, *rect))
	}
	#[inline]
	fn texture(&self, texture: &Self::Texture, rect: &Rect<f32>) {
		self.push(Command::Texture(texture.clone(), *rect))
	}
	#[inline]
	fn texture_frame(&self, texture: &Self::Texture, rect: &Rect<f32>, frame: &Rect<f32>) {
		self.push(Command::TextureFrame(texture.clone(), *rect, *frame))
	}
	#[inline]
	fn measure_text(&self, text: &str) -> Vector2<f32> {
		let scale = rusttype::Scale::uniform(self.font_size);
		let p = self.font.layout(text, scale, rusttype::point(0.0, 0.0))
			.filter_map(|g| g.pixel_bounding_box())
			.map(|bb| Rect::from_coords(bb.min.x, bb.min.y, bb.max.x, bb.max.y))
			.fold(Rect::default(), |rect, bb| rect.union_with_empty(bb));

		let vm = self.font.v_metrics(scale);

		let w = p.dx();
		let h = vm.ascent + vm.descent;

		debug_assert!(w >= 0);

		Vector2::new(w as f32, h as f32)
	}
	#[inline]
	fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
		self.push(Command::Text(base, color, text.to_string()))
	}

	#[inline]
	fn custom(&self, cmd: Self::Custom) {
		self.push(Command::Custom(cmd))
	}
}

#[derive(Clone)]
pub struct VertexCtor {
	pub proj: Affine<f32>,
	pub color: [u8; 4],
}

impl VertexConstructor<FillVertex, GpuFillVertex> for VertexCtor {
	fn new_vertex(&mut self, vertex: FillVertex) -> GpuFillVertex {
		debug_assert!(!vertex.position.x.is_nan());
		debug_assert!(!vertex.position.y.is_nan());
		let position = Vector2::new(vertex.position.x, vertex.position.y);
		let position = self.proj.transform_vector(position).into();
		GpuFillVertex {
			position,
			color: self.color,
		}
	}
}

impl VertexConstructor<StrokeVertex, GpuFillVertex> for VertexCtor {
	fn new_vertex(&mut self, vertex: StrokeVertex) -> GpuFillVertex {
		debug_assert!(!vertex.position.x.is_nan());
		debug_assert!(!vertex.position.y.is_nan());
		let position = Vector2::new(vertex.position.x, vertex.position.y);
		let position = self.proj.transform_vector(position).into();
		GpuFillVertex {
			position,
			color: self.color,
		}
	}
}
