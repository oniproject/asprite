use common::*;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::gfx::primitives::ToColor;
use sdl2::pixels::Color;

use sdl2;

pub enum Align {
	Left,
	Right,
	Center,
}

pub enum Event {
	Press,
	Release,

	MouseEnter,
	MouseLeave,
}

pub trait Render {
	fn pixel(&self, p: Point<i16>, color: u32);
	fn line(&self, start: Point<i16>, end: Point<i16>, color: u32);

	fn rect(&self, r: Rect<i16>, color: u32);
	fn outline(&self, r: Rect<i16>, color: u32);

	fn icon(&self, r: Rect<i16>, index: usize);
	fn text(&mut self, r: Rect<i16>, align: Align, color: u32, s: &str);

	//fn bezier(&self, vx: &[i16], vy: &[i16], s: i32, color: u32);
}

#[derive(PartialEq)]
pub enum Key {
	NextWidget,
	PrevWidget,
}

pub struct RenderSDL<'ttf_module, 'rwops, T: sdl2::render::RenderTarget> {
	pub ctx: sdl2::render::Canvas<T>,
	pub font: sdl2::ttf::Font<'ttf_module, 'rwops>,

	pub hot: u32,
	pub active: u32,
	pub kbd: u32,
	pub last: u32,

	pub key: Option<Key>,
	pub mouse: (bool, Point<i16>),
}

impl<'ttf_module, 'rwops> RenderSDL<'ttf_module, 'rwops, sdl2::video::Window> {
	pub fn label(&mut self, id: u32, r: Rect<i16>, align: Align, color: u32, text: &str) {
		self.widget(id, r);
		self.text(r, align, color, text);
	}
	pub fn label_bg(&mut self, id: u32, r: Rect<i16>, align: Align, color: u32, bg: u32, text: &str) {
		self.widget(id, r);
		self.rect(r, bg);
		self.text(r, align, color, text);
	}
	pub fn btn_label<F: FnOnce()>(&mut self, id: u32, r: Rect<i16>, label: &str, cb: F) {
		self.widget(id, r);

		let bg = 0x0074D9_FF;
		let active_color = 0x7FDBFF_FF;

		let label_color = if self.hot == id {
			let bg = if self.active == id { active_color } else { bg };
			self.rect(r, bg);
			0x222222_FF
		} else {
			0xFFFFFF_FF
		};
		self.outline(r, bg);
		self.text(r, Align::Center, label_color, label);

		if !self.mouse.0 && self.hot == id && self.active == id {
			println!("click: {}", label);
			cb();
		}
	}

	pub fn prepare(&mut self) {
		self.hot = 0;
	}

	pub fn finish(&mut self) {
		if !self.mouse.0 {
			self.active = 0;
		} else if self.active == 0 {
			self.active = 0xFFFFFF;
		}

		// If no widget grabbed tab, clear focus
		if self.key == Some(Key::NextWidget) {
			self.kbd = 0;
		}

		// Clear the entered key
		self.key = None;
	}

	pub fn widget(&mut self, id: u32, r: Rect<i16>) {
		if self.hot == 0 && r.contains(self.mouse.1) {
			self.hot = id;
			if self.active == 0 && self.mouse.0 {
				self.active = id;
			}
		}
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
		self.last = id;
	}
}

//impl<'a, T: sdl2::render::RenderTarget + 'a> widget::Render for Render<'a, T> {
impl<'ttf_module, 'rwops> Render for RenderSDL<'ttf_module, 'rwops, sdl2::video::Window> {
	fn pixel(&self, p: Point<i16>, color: u32) {
		self.ctx.pixel(p.x, p.y, color.to_be()).unwrap()
	}
	fn line(&self, start: Point<i16>, end: Point<i16>, color: u32) {
		self.ctx.line(
			start.x, start.y,
			end.x, end.y,
			color.to_be()).unwrap()
	}

	fn rect(&self, r: Rect<i16>, color: u32) {
		self.ctx.box_(
			r.min.x, r.min.y,
			r.max.x-1, r.max.y-1,
			color.to_be()).unwrap()
	}

	fn outline(&self, r: Rect<i16>, color: u32) {
		self.ctx.rectangle(
			r.min.x, r.min.y,
			r.max.x, r.max.y,
			color.to_be()).unwrap()

	}

	fn icon(&self, _: Rect<i16>, _: usize) {
		unimplemented!()
	}

	fn text(&mut self, r: Rect<i16>, align: Align, color: u32, s: &str) {
		let color = {
			let (r, g, b, a) = color.to_be().as_rgba();
			Color::RGBA(r, g, b, a)
		};
		// render a surface, and convert it to a texture bound to the canvas
		let surface = self.font
			.render(s)
			.blended(color)
			.unwrap();

		let creator = self.ctx.texture_creator();

		let texture = creator
			.create_texture_from_surface(&surface)
			.unwrap();

		let sdl2::render::TextureQuery { width, height, .. } = texture.query();
		let rw = r.max.x - r.min.x;
		let rh = r.max.y - r.min.y;

		let addx = match align {
			Align::Left => 0,
			Align::Center => (rw - width as i16) / 2,
			Align::Right => rw - width as i16,
		};
		let addy = (rh - height as i16) / 2;

		let r = sdl2::rect::Rect::new(
			(r.min.x + addx) as i32,
			(r.min.y + addy) as i32, width, height);

		self.ctx
			.copy(&texture, None, Some(r))
			.unwrap();
	}

	//fn bezier(&self, vx: &[i16], vy: &[i16], s: i32, color: u32);
}
