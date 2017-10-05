mod im;
mod theme;
mod graphics;

pub use self::im::*;
pub use self::theme::*;
pub use self::graphics::*;

use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, TextureQuery, Texture, TextureCreator, BlendMode};
use sdl2::ttf::Font;
use sdl2::pixels::PixelFormatEnum;
use sdl2::mouse::{Cursor, SystemCursor};
use sdl2::image::LoadTexture;
pub use sdl2::gfx::primitives::DrawRenderer;

use std::path::Path;
use std::collections::HashMap;

use common::*;

type Cursors = HashMap<SystemCursor, Cursor>;
fn create_cursors() -> Cursors {
	let cursors = [
		SystemCursor::Arrow,
		SystemCursor::IBeam,
		SystemCursor::Wait,
		SystemCursor::Crosshair,
		SystemCursor::WaitArrow,
		SystemCursor::SizeNWSE,
		SystemCursor::SizeNESW,
		SystemCursor::SizeWE,
		SystemCursor::SizeNS,
		SystemCursor::SizeAll,
		SystemCursor::No,
		SystemCursor::Hand,
	];

	let cursors: HashMap<_, _> = cursors.iter().map(|&c| (c, Cursor::from_system(c).unwrap())).collect();
	cursors[&SystemCursor::Crosshair].set();
	cursors
}


#[derive(PartialEq)]
pub enum Key {
	NextWidget,
	PrevWidget,
}

pub struct Render<'t, 'ttf_module, 'rwops> {
	pub ctx: Canvas<Window>,
	pub font: Font<'ttf_module, 'rwops>,

	pub creator: &'t TextureCreator<WindowContext>,

	pub textures: HashMap<usize, (Texture<'t>, u32, u32)>,
	pub last_texture_id: usize,

	pub hot: u32,
	pub active: u32,
	//pub kbd: u32,
	pub last: u32,

	pub next_rect: Rect<i16>,

	pub key: Option<Key>,
	pub mouse: (bool, Point<i16>),

	cursors: Cursors,

	channel: usize,
	cmd_buffer: Vec<Vec<Command<i16, u32>>>,
}

impl<'t, 'ttf, 'rwops> Render<'t, 'ttf, 'rwops> {
	pub fn new(ctx: Canvas<Window>, creator: &'t TextureCreator<WindowContext>, font: Font<'ttf, 'rwops>) -> Self {
		Self {
			ctx, font, creator,
			last: 0, hot: 0, active: 0, // kbd: 0,
			key: None,
			next_rect: Rect::with_coords(0, 0, 0, 0),
			mouse: (false, Point::new(0, 0)),
			textures: HashMap::new(),
			last_texture_id: 0,
			cursors: create_cursors(),
			channel: 0,
			cmd_buffer: vec![Vec::new()],
		}
	}

	pub fn load_texture<T: Into<Option<usize>>, P: AsRef<Path>>(&mut self, id: T, filename: P) -> usize {
		let id = id.into().unwrap_or_else(|| {
			let id = self.last_texture_id;
			self.last_texture_id += 1;
			id
		});

		let mut texture = self.creator.load_texture(filename).unwrap();
		texture.set_blend_mode(BlendMode::Blend);
		let TextureQuery { width, height, .. } = texture.query();
		self.textures.insert(id, (texture, width, height));
		id
	}

	pub fn create_texture<T: Into<Option<usize>>>(&mut self, id: T, w: u32, h: u32) -> usize {
		let id = id.into().unwrap_or_else(|| {
			let id = self.last_texture_id;
			self.last_texture_id += 1;
			id
		});

		let mut texture = self.creator.create_texture_target(PixelFormatEnum::RGBA8888, w, h).unwrap();
		texture.set_blend_mode(BlendMode::Blend);
		let TextureQuery { width, height, .. } = texture.query();
		self.textures.insert(id, (texture, width, height));
		id
	}

	pub fn prepare(&mut self) {
		self.hot = 0;
		self.channel = 0;
	}

	pub fn finish(&mut self) -> bool {
		let last_active = self.active;
		//let last_kbd = self.kbd;

		if !self.mouse.0 {
			self.active = 0;
		} else if self.active == 0 {
			self.active = 0xFFFFFF;
		}

		// If no widget grabbed tab, clear focus
		/*if self.key == Some(Key::NextWidget) {
			self.kbd = 0;
		}*/

		// Clear the entered key
		self.key = None;

		let cur = if self.hot != 0 { SystemCursor::Hand } else { SystemCursor::Crosshair };
		self.cursors[&cur].set();

		self.run_commands();

		last_active != self.active //|| last_kbd != self.kbd
	}

	fn run_commands(&mut self) {
		for buf in &mut self.cmd_buffer {
		for cmd in buf.drain(..) {
		match cmd {
			Command::Line(start, end, color) =>
				self.ctx.line(
					start.x, start.y,
					end.x, end.y,
					color.to_be()).unwrap(),

			Command::Fill(r, color) =>
				self.ctx.box_(
					r.min.x, r.min.y,
					r.max.x-1, r.max.y-1,
					color.to_be()).unwrap(),
			Command::Border(mut r, color) => {
				r.max.x -= 1;
				r.max.y -= 1;
				let color = color.to_be();
				let (x1, x2) = (r.min.x, r.max.x);
				self.ctx.hline(x1, x2, r.min.y, color).unwrap();
				self.ctx.hline(x1, x2, r.max.y, color).unwrap();
				let (y1, y2) = (r.min.y, r.max.y);
				self.ctx.vline(r.min.x, y1, y2, color).unwrap();
				self.ctx.vline(r.max.x, y1, y2, color).unwrap();
			}

			Command::Image(id, r) => {
				let &(ref texture, w, h) = &self.textures[&id];
				let dx = (r.dx() as i32 - w as i32) / 2;
				let dy = (r.dy() as i32 - h as i32) / 2;
				let (x, y) = (r.min.x as i32 + dx, r.min.y as i32 + dy);
				let dst = rect!(x, y, w, h);
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
}

impl<'t, 'ttf, 'rwops> Graphics<i16, u32> for Render<'t, 'ttf, 'rwops> {
	fn command(&mut self, cmd: Command<i16, u32>) {
		self.cmd_buffer[self.channel].push(cmd);
	}
	fn text_size(&mut self, s: &str) -> (u32, u32) {
		let (w, h) = self.font.size_of(s).unwrap();
		(w - 1, h - 1)
	}
	fn channel(&mut self, ch: usize) { self.channel = ch }
}

impl<'t, 'ttf, 'rwops> Immediate for Render<'t, 'ttf, 'rwops> {
	fn bounds(&self) -> Rect<i16> {
		let size = self.ctx.window().size();
		Rect::with_size(0, 0, size.0 as i16, size.1 as i16)
	}

	fn widget_rect(&self) -> Rect<i16> {
		self.next_rect
	}
	fn widget(&mut self, id: u32) -> Rect<i16> {
		if self.hot == 0 && self.next_rect.contains(self.mouse.1) {
			self.hot = id;
			if self.active == 0 && self.mouse.0 {
				self.active = id;
			}
		}
		/*
		if self.kbd == 0 {
			self.kbd = id;
		}
		if self.kbd == id {
			let (reset, kbd) = match self.key {
				None => (false, id),
				Some(Key::NextWidget) => (true, 0),
				Some(Key::PrevWidget) => (true, self.last),
			};

			if reset {
				self.key = None;
			}
			self.kbd = kbd;
		}
		*/
		self.last = id;
		self.next_rect
	}

	fn lay(&mut self, r: Rect<i16>) {
		self.next_rect = r;
	}

	fn is_hot(&self) -> bool {
		self.hot == self.last
	}
	fn is_active(&self) -> bool {
		self.active == self.last
	}

	fn is_click(&self) -> bool {
		!self.mouse.0 && self.hot == self.last && self.active == self.last
	}
}