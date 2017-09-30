use sdl2;
use sdl2::video::Window;
use sdl2::render::{RenderTarget, Canvas};
use sdl2::ttf::Font;
use sdl2::gfx::primitives::{DrawRenderer, ToColor};
use sdl2::pixels::Color;

use common::*;

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

pub enum RenderCommand<N: Signed, C: Copy> {
	Line(Point<N>, Point<N>, C),
	Border(Rect<N>, C),
	Fill(Rect<N>, C),
	Clip(Option<Rect<N>>),
	Text(String, Point<N>, C),
	Image(usize, Rect<N>),
}

pub trait Render<N: Signed, C: Copy> {
	// TODO: images

	fn command(&mut self, cmd: RenderCommand<N, C>);
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
		self.command(RenderCommand::Line(a, b, color));
	}
	fn border(&mut self, r: Rect<N>, color: C) {
		self.command(RenderCommand::Border(r, color));
	}
	fn fill(&mut self, r: Rect<N>, color: C) {
		self.command(RenderCommand::Fill(r, color));
	}
	fn clip(&mut self, r: Option<Rect<N>>) {
		self.command(RenderCommand::Clip(r));
	}
	fn image(&mut self, m: usize, r: Rect<N>) {
		self.command(RenderCommand::Image(m, r));
	}
	fn text(&mut self, p: Point<N>, color: C, s: &str) {
		self.command(RenderCommand::Text(s.to_string(), p, color));
	}
}

pub trait WidgetRender: Render<i16, u32> {
	fn widget(&mut self, id: u32) -> Rect<i16>;
	fn widget_rect(&self) -> Rect<i16>;

	// fn bounds(&self) -> Rect<i16>;

	fn is_hot(&self) -> bool;
	fn is_active(&self) -> bool;
	fn is_click(&self) -> bool;

	fn lay(&mut self, r: Rect<i16>);
}

#[derive(PartialEq)]
pub enum Key {
	NextWidget,
	PrevWidget,
}

pub struct Panel<'a, R: WidgetRender + 'a> {
	pub render: &'a mut R,
	pub r: Rect<i16>,
}

impl<'a, R: WidgetRender + 'a> Panel<'a, R> {
	pub fn run<F: FnOnce(Self)>(self, f: F) { f(self) }

	pub fn panel(&mut self, r: Rect<i16>) -> Panel<Self> {
		Panel {
			render: self,
			r,
		}
	}

	pub fn width(&self) -> i16 {
		self.r.w()
	}
	pub fn height(&self) -> i16 {
		self.r.h()
	}

	pub fn clear(&mut self, color: u32) {
		let r = self.r;
		self.lay(Rect::with_size(0, 0, r.w(), r.h()));
		self.render.fill(r, color);
	}

	pub fn frame<A, B>(&mut self, bg: A, border: B)
		where
			A: Into<Option<u32>>,
			B: Into<Option<u32>>,
	{
		let r = self.widget_rect();
		if let Some(bg) = bg.into() {
			self.fill(r, bg);
		}
		if let Some(border) = border.into() {
			self.border(r, border);
		}
	}

	pub fn label_right(&mut self, color: u32, text: &str) {
		self.label(Align::Right, color, text)
	}
	pub fn label_center(&mut self, color: u32, text: &str) {
		self.label(Align::Center, color, text)
	}
	pub fn label_left(&mut self, color: u32, text: &str) {
		self.label(Align::Left, color, text)
	}

	fn label(&mut self, align: Align, color: u32, text: &str) {
		let r = self.widget_rect();
		match align {
			Align::Left => self.text_center_left(r, color, text),
			Align::Right => self.text_center_right(r, color, text),
			Align::Center => self.text_center(r, color, text),
		}
	}
	fn label_bg(&mut self, align: Align, color: u32, bg: u32, text: &str) {
		let r = self.widget_rect();
		self.fill(r, bg);
		match align {
			Align::Left => self.text_center_left(r, color, text),
			Align::Right => self.text_center_right(r, color, text),
			Align::Center => self.text_center(r, color, text),
		}
	}

	pub fn btn_color(&mut self, id: u32, color: u32) -> bool {
		let r = self.widget(id);
		self.fill(r, color);
		self.is_click()
	}

	pub fn btn_mini<F: FnMut()>(&mut self, id: u32, label: &str, active: u32, mut cb: F) {
		let r = self.widget(id);

		if self.is_hot() && self.is_active() {
			self.fill(r, active);
		};

		let label_color = 0xFFFFFF_FF;

		// FIXME: fucking hack
		{
			let mut r = r.clone();
			let w = r.w() + 2;
			let h = r.h() + 2;
			r.set_w(w);
			r.set_h(h);
			self.text_center(r, label_color, label);
		}

		if self.is_click() {
			cb();
			println!("click: {}", label);
		}
	}

	pub fn btn_label<F: FnMut()>(&mut self, id: u32, label: &str, mut cb: F) {
		let r = self.widget(id);

		let bg = 0x353D4B_FF;
		let active_color = 0x0076FF_FF;

		if self.is_hot() {
			let bg = if self.is_active() { active_color } else { bg };
			self.fill(r, bg);
		};
		self.border(r, bg);

		let label_color = 0xECECEC_FF;

		// FIXME: fucking hack
		{
			let mut r = r.clone();
			let w = r.w() + 2;
			let h = r.h() + 2;
			r.set_w(w);
			r.set_h(h);
			self.text_center(r, label_color, label);
		}

		if self.is_click() {
			cb();
			println!("click: {}", label);
		}
	}
}

impl<'a, R: WidgetRender + 'a> Render<i16, u32> for Panel<'a, R> {
	fn command(&mut self, cmd: RenderCommand<i16, u32>) { self.render.command(cmd) }
	fn text_size(&mut self, s: &str) -> (u32, u32) { self.render.text_size(s) }
}

impl<'a, R: WidgetRender + 'a> WidgetRender for Panel<'a, R> {
	fn widget(&mut self, id: u32) -> Rect<i16> { self.render.widget(id) }

	fn widget_rect(&self) -> Rect<i16> { self.render.widget_rect() }

	fn is_hot(&self) -> bool { self.render.is_hot() }
	fn is_active(&self) -> bool { self.render.is_active() }
	fn is_click(&self) -> bool { self.render.is_click() }
	fn lay(&mut self, r: Rect<i16>) {
		let r = self.r.min_translate_rect(r);
		self.render.lay(r);
	}
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

	pub fn panel(&mut self, r: Rect<i16>) -> Panel<Self> {
		Panel {
			render: self,
			r,
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

impl<'ttf, 'rwops> Render<i16, u32> for RenderSDL<'ttf, 'rwops, Window> {
	fn command(&mut self, cmd: RenderCommand<i16, u32>) {
		match cmd {
			RenderCommand::Line(start, end, color) =>
				self.ctx.line(
					start.x, start.y,
					end.x, end.y,
					color.to_be()).unwrap(),

			RenderCommand::Fill(r, color) =>
				self.ctx.box_(
					r.min.x, r.min.y,
					r.max.x, r.max.y-1,
					color.to_be()).unwrap(),
			RenderCommand::Border(r, color) => {
				let color = color.to_be();
				let (x1, x2) = (r.min.x, r.max.x);
				self.ctx.hline(x1, x2, r.min.y, color).unwrap();
				self.ctx.hline(x1, x2, r.max.y, color).unwrap();
				let (y1, y2) = (r.min.y, r.max.y);
				self.ctx.vline(r.min.x, y1, y2, color).unwrap();
				self.ctx.vline(r.max.x, y1, y2, color).unwrap();
			}

			RenderCommand::Image(_id, _r) => unimplemented!(),
			RenderCommand::Clip(_r) => unimplemented!(),
			RenderCommand::Text(s, p, color) => {
				let color = {
					let (r, g, b, a) = color.to_be().as_rgba();
					Color::RGBA(r, g, b, a)
				};

				// render a surface, and convert it to a texture bound to the canvas
				let surface = self.font
					.render(&s)
					.blended(color)
					.unwrap();

				let creator = self.ctx.texture_creator();

				let texture = creator
					.create_texture_from_surface(&surface)
					.unwrap();

				let sdl2::render::TextureQuery { width, height, .. } = texture.query();


				let r = sdl2::rect::Rect::new(p.x as i32, p.y as i32, width, height);

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

impl<'ttf, 'rwops> WidgetRender for RenderSDL<'ttf, 'rwops, Window> {
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
