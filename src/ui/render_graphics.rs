use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, TextureQuery, Texture, TextureCreator, BlendMode};
use sdl2::ttf::Font;
use sdl2::pixels::PixelFormatEnum;
use sdl2::image::LoadTexture;
pub use sdl2::gfx::primitives::DrawRenderer;

use std::path::Path;
use std::collections::HashMap;

use common::*;
use super::*;

pub struct RenderGraphics<'t, 'ttf_module, 'rwops, N: SignedInt, C: Copy + 'static> {
	pub ctx: Canvas<Window>,
	pub font: Font<'ttf_module, 'rwops>,
	pub creator: &'t TextureCreator<WindowContext>,
	pub textures: HashMap<usize, (Texture<'t>, u32, u32)>,
	pub last_texture_id: usize,

	pub channel: usize,
	cmd_buffer: Vec<Vec<Command<N, C>>>,
}

impl<'t, 'ttf, 'rwops, N: SignedInt, C: Copy + 'static>  RenderGraphics<'t, 'ttf, 'rwops, N, C> {
	fn gen_id<T: Into<Option<usize>>>(&mut self, id: T) -> usize {
		id.into().unwrap_or_else(|| {
			let id = self.last_texture_id;
			self.last_texture_id += 1;
			id
		})
	}
}

impl<'t, 'ttf, 'rwops> RenderGraphics<'t, 'ttf, 'rwops, i16, u32> {
	pub fn new(ctx: Canvas<Window>, creator: &'t TextureCreator<WindowContext>, font: Font<'ttf, 'rwops>) -> Self {
		Self {
			ctx, font, creator,
			
			textures: HashMap::new(),
			last_texture_id: 0,
			channel: 0,
			cmd_buffer: vec![Vec::new()],
		}
	}

	pub fn load_texture<T: Into<Option<usize>>, P: AsRef<Path>>(&mut self, id: T, filename: P) -> usize {
		let id = self.gen_id(id);

		let mut texture = self.creator.load_texture(filename).unwrap();
		texture.set_blend_mode(BlendMode::Blend);
		let TextureQuery { width, height, .. } = texture.query();
		self.textures.insert(id, (texture, width, height));
		id
	}

	pub fn run_commands(&mut self) {
		for buf in &mut self.cmd_buffer {
			for cmd in buf.drain(..) {
				match cmd {
				Command::Line(start, end, color) => Self::line(&self.ctx, start, end, color),
				Command::Fill(r, color) => Self::rect_filled(&self.ctx, r, color),
				Command::Border(r, color) => Self::rect(&self.ctx, r, color),

				Command::Image(id, p, zoom) => {
					let &(ref texture, w, h) = &self.textures[&id];
					let zoom = zoom as u32;
					let dst = rect!(p.x, p.y, w * zoom, h * zoom);
					self.ctx.copy(texture, None, dst).unwrap();
				}

				Command::Clip(r) => {
					self.ctx.set_clip_rect(r.map(|r|
						rect!(r.min.x, r.min.y, r.dx(), r.dy())
					));
				}

				Command::Text(s, p, color) => {
					let color = color!(color);

					// render a surface, and convert it to a texture bound to the canvas
					let surface = self.font.render(&s).blended(color).unwrap();
					let creator = self.ctx.texture_creator();
					let texture = creator
						.create_texture_from_surface(&surface)
						.unwrap();

					let TextureQuery { width, height, .. } = texture.query();
					let r = rect!(p.x, p.y, width, height);

					self.ctx
						.copy(&texture, None, Some(r))
						.unwrap();
					}
				}
			}
		}
	}

	fn line(ctx: &Canvas<Window>, start: Point<i16>, end: Point<i16>, color: u32) {
		ctx.line(
			start.x, start.y,
			end.x, end.y,
			color.to_be()).unwrap()
	}

	fn rect_filled(ctx: &Canvas<Window>, r: Rect<i16>, color: u32) {
		ctx.box_(
			r.min.x, r.min.y,
			r.max.x-1, r.max.y-1,
			color.to_be()).unwrap()
	}

	fn rect(ctx: &Canvas<Window>, mut r: Rect<i16>, color: u32) {
		r.max.x -= 1;
		r.max.y -= 1;

		let color = color.to_be();
		let (x1, x2) = (r.min.x, r.max.x);
		ctx.hline(x1, x2, r.min.y, color).unwrap();
		ctx.hline(x1, x2, r.max.y, color).unwrap();
		let (y1, y2) = (r.min.y, r.max.y);
		ctx.vline(r.min.x, y1, y2, color).unwrap();
		ctx.vline(r.max.x, y1, y2, color).unwrap();
	}
}

impl<'t, 'ttf, 'rwops, N: SignedInt, C: Copy + 'static> widgets::Graphics<N, C> for RenderGraphics<'t, 'ttf, 'rwops, N, C> {
	fn render_text_center(&mut self, r: Rect<N>, color: C, s: &str) {
		self.text_center(r, color, s);
	}
	fn render_rect(&mut self, r: Rect<N>, color: C) {
		self.fill(r, color);
	}
	fn render_border(&mut self, r: Rect<N>, color: C) {
		self.border(r, color);
	}
}

impl<'t, 'ttf, 'rwops, N: SignedInt, C: Copy + 'static> Graphics<N, C> for RenderGraphics<'t, 'ttf, 'rwops, N, C> {
	type RenderTarget = SdlCanvas;

	fn canvas<F: FnMut(&mut Self::RenderTarget, u32, u32)>(&mut self, id: usize, mut f: F) {
		let texture = self.textures.get_mut(&id);
		if let Some(texture) = texture {
			let w = texture.1;
			let h = texture.2;
			self.ctx.with_texture_canvas(&mut texture.0, |canvas| f(canvas, w, h)).unwrap();
		}
	}

	fn command(&mut self, cmd: Command<N, C>) {
		self.cmd_buffer[self.channel].push(cmd);
	}
	fn text_size(&mut self, s: &str) -> (u32, u32) {
		let (w, h) = self.font.size_of(s).unwrap();
		(w - 1, h - 1)
	}
	fn image_size(&mut self, id: usize) -> (u32, u32) {
		let (_, w, h) = self.textures[&id];
		(w, h)
	}
	fn channel(&mut self, ch: usize) { self.channel = ch }

	fn create_texture<T: Into<Option<usize>>>(&mut self, id: T, w: u32, h: u32) -> usize {
		let id = self.gen_id(id);

		let mut texture = self.creator.create_texture_target(PixelFormatEnum::RGBA8888, w, h).unwrap();
		texture.set_blend_mode(BlendMode::Blend);
		let TextureQuery { width, height, .. } = texture.query();
		self.textures.insert(id, (texture, width, height));
		id
	}
}