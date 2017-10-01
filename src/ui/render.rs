use sdl2::video::Window;
use sdl2::render::{RenderTarget, Canvas, TextureQuery};
use sdl2::ttf::Font;
use sdl2::gfx::primitives::{DrawRenderer, ToColor};
use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;

use common::*;
use super::im::*;

pub enum Event {
	Press,
	Release,

	MouseEnter,
	MouseLeave,
}

pub enum Command<'a, N: Signed, C: Copy> {
	Line(Point<N>, Point<N>, C),
	Border(Rect<N>, C),
	Fill(Rect<N>, C),
	Clip(Option<Rect<N>>),
	Text(&'a str, Point<N>, C),
	Image(usize, Rect<N>),
}

pub trait Graphics<N: Signed, C: Copy> {
	// TODO: images

	fn command<'a>(&mut self, cmd: Command<'a, N, C>);
	fn text_size(&mut self, s: &str) -> (u32, u32);

	fn text_center_left(&mut self, r: Rect<N>, color: C, s: &str) {
		self.text_align(r, 0.0, 0.5, color, s);
	}

	fn text_center_right(&mut self, r: Rect<N>, color: C, s: &str) {
		self.text_align(r, 1.0, 0.5, color, s);
	}

	fn text_center(&mut self, r: Rect<N>, color: C, s: &str) {
		self.text_align(r, 0.5, 0.5, color, s);
	}

	fn text_align(&mut self, r: Rect<N>, x: f32, y: f32, color: C, s: &str) {
		let (tw, th) = self.text_size(s);
		let (tw, th) = (N::from(tw).unwrap(), N::from(th).unwrap());
		let (rw, rh) = (r.w(), r.h());

		let dw = (rw - tw).to_f32().unwrap();
		let dh = (rh - th).to_f32().unwrap();

		let x = r.min.x + N::from(dw * x).unwrap();
		let y = r.min.y + N::from(dh * y).unwrap();

		let p = Point::new(x, y);
		self.text(p, color, s);
	}

	fn line(&mut self, a: Point<N>, b: Point<N>, color: C) {
		self.command(Command::Line(a, b, color));
	}
	fn border(&mut self, r: Rect<N>, color: C) {
		self.command(Command::Border(r, color));
	}
	fn fill(&mut self, r: Rect<N>, color: C) {
		self.command(Command::Fill(r, color));
	}
	fn clip(&mut self, r: Option<Rect<N>>) {
		self.command(Command::Clip(r));
	}
	fn image(&mut self, m: usize, r: Rect<N>) {
		self.command(Command::Image(m, r));
	}
	fn text(&mut self, p: Point<N>, color: C, s: &str) {
		self.command(Command::Text(s, p, color));
	}
}


#[derive(PartialEq)]
pub enum Key {
	NextWidget,
	PrevWidget,
}

pub struct RenderSDL<'ttf_module, 'rwops, T: RenderTarget> {
	pub ctx: Canvas<T>,
	pub font: Font<'ttf_module, 'rwops>,

	pub hot: u32,
	pub active: u32,
	//pub kbd: u32,
	pub last: u32,

	pub next_rect: Rect<i16>,

	pub key: Option<Key>,
	pub mouse: (bool, Point<i16>),
}

impl<'ttf, 'rwops> RenderSDL<'ttf, 'rwops, Window> {
	pub fn new(ctx: Canvas<Window>, font: Font<'ttf, 'rwops>) -> Self {
		Self {
			ctx, font,
			last: 0, hot: 0, active: 0, // kbd: 0,
			key: None,
			next_rect: Rect::with_coords(0, 0, 0, 0),
			mouse: (false, Point::new(0, 0)),
		}
	}


	pub fn prepare(&mut self) {
		self.hot = 0;
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

		last_active != self.active //|| last_kbd != self.kbd
	}

}

impl<'ttf, 'rwops> Graphics<i16, u32> for RenderSDL<'ttf, 'rwops, Window> {
	fn command(&mut self, cmd: Command<i16, u32>) {
		match cmd {
			Command::Line(start, end, color) =>
				self.ctx.line(
					start.x, start.y,
					end.x, end.y,
					color.to_be()).unwrap(),

			Command::Fill(r, color) =>
				self.ctx.box_(
					r.min.x, r.min.y,
					r.max.x, r.max.y-1,
					color.to_be()).unwrap(),
			Command::Border(r, color) => {
				let color = color.to_be();
				let (x1, x2) = (r.min.x, r.max.x);
				self.ctx.hline(x1, x2, r.min.y, color).unwrap();
				self.ctx.hline(x1, x2, r.max.y, color).unwrap();
				let (y1, y2) = (r.min.y, r.max.y);
				self.ctx.vline(r.min.x, y1, y2, color).unwrap();
				self.ctx.vline(r.max.x, y1, y2, color).unwrap();
			}

			Command::Image(_id, _r) => unimplemented!(),
			Command::Clip(_r) => unimplemented!(),
			Command::Text(s, p, color) => {
				let color = {
					let (r, g, b, a) = color.to_be().as_rgba();
					Color::RGBA(r, g, b, a)
				};

				// render a surface, and convert it to a texture bound to the canvas
				let surface = self.font.render(s).blended(color).unwrap();
				let creator = self.ctx.texture_creator();
				let texture = creator
					.create_texture_from_surface(&surface)
					.unwrap();

				let TextureQuery { width, height, .. } = texture.query();
				let r = SdlRect::new(p.x as i32, p.y as i32, width, height);

				self.ctx
					.copy(&texture, None, Some(r))
					.unwrap();
			}
		}
	}
	fn text_size(&mut self, s: &str) -> (u32, u32) {
		self.font.size_of(s).unwrap()
	}
}

impl<'ttf, 'rwops> Immediate for RenderSDL<'ttf, 'rwops, Window> {
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
