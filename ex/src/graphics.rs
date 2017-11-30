use vulkano::command_buffer::AutoCommandBufferBuilder;

use lyon::path::{Path, Builder};
use lyon::path_builder::*;
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

type Canvas = SvgPathBuilder<Builder>;

enum Command {
	Quad([u8; 4], Rect<f32>),
	Texture(Texture, Rect<f32>),
	TextureFrame(Texture, Rect<f32>, Rect<f32>),
	Text(Point2<f32>, [u8; 4], String),

	Fill([u8;4], Affine<f32>, Path),
}

pub struct Graphics {
	buffer: Mutex<Vec<Command>>,
	pub font: Font<'static>,
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
			buffer: Mutex::new(Vec::new()),
			hovered: AtomicBool::new(false),
			vg: Mutex::new(VertexBuffers::new()),
			font,
			font_size,
		}
	}

	#[inline(always)]
	fn buf(&self) -> MutexGuard<Vec<Command>> {
		self.buffer.lock().unwrap()
	}

	#[inline(always)]
	fn push(&self, cmd: Command)  {
		self.buf().push(cmd)
	}

	pub fn is_hovered(&self) -> bool {
		self.hovered.load(Ordering::Relaxed)
	}

	pub fn run(&self, mut cb: AutoCommandBufferBuilder, renderer: &mut Renderer) -> errors::Result<AutoCommandBufferBuilder> {
		self.hovered.store(false, Ordering::Relaxed);

		const COLOR: [u8; 4] = [0xFF; 4];
		const UV: [[u16; 2]; 4] = zero_uv();

		let mut sprite = false;
		let mut buf = self.buf();
		let mut vg = self.vg.lock().unwrap();
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
			Command::Text(base, c, s) => {
				let size = rusttype::Scale::uniform(self.font_size);
				let v = self.font.v_metrics(size);
				let mut base = base + Vector2::new(0.0, v.ascent + v.descent);
				base.x = base.x.trunc() + 0.5;
				base.y = base.y.trunc() + 0.5;

				// TODO: reduce reallocations
				let lay: Vec<_> = renderer.text_lay(&self.font, self.font_size, &s, base.x, base.y).collect();

				cb = if !sprite { cb } else { sprite = false; renderer.end_sprites(cb)? };
				cb = renderer.glyphs(cb, &lay, c)?;
			}

			Command::Fill(color, proj, path) => {
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
				cb = renderer.end_vg(cb)?
			}
			}
		}

		Ok(if !sprite { cb } else { renderer.end_sprites(cb)? })
	}
}

impl ui::Graphics for Graphics {
	type Texture = Texture;
	type Color = [u8; 4];
	type Path = Path;

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
		let size = rusttype::Scale::uniform(self.font_size);
		let p = self.font.layout(text, size, rusttype::point(0.0, 0.0))
			.filter_map(|g| g.pixel_bounding_box())
			.map(|bb| Rect::with_coords(bb.min.x, bb.min.y, bb.max.x, bb.max.y))
			.fold(Rect::new(), |rect, bb|
				rect.union_raw(bb) //.intersect(Rect::with_size(-10, -s, 300, s * 2))
			);

		let w = p.dx();
		let h = p.dy();
		assert!(w > 0 && h > 0);

		Vector2::new(w as f32, h as f32)
	}
	#[inline]
	fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
		self.push(Command::Text(base, color, text.to_string()))
	}

	fn fill(&self, color: Self::Color, proj: Affine<f32>, path: Path) {
		self.push(Command::Fill(color, proj, path))
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
