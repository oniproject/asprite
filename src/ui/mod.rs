mod im;
mod theme;
mod graphics;
mod widgets;
mod event;
mod render_graphics;

pub use self::render_graphics::*;

pub use self::im::*;
pub use self::theme::*;
pub use self::graphics::*;

use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::mouse::{Cursor, SystemCursor};
pub use sdl2::gfx::primitives::DrawRenderer;

use std::collections::HashMap;

use common::*;

pub type SdlCanvas = Canvas<Window>;

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

#[derive(Clone)]
pub enum Mouse<N: SignedInt> {
	Move(Point<N>),
	Press(Point<N>),
	Release(Point<N>),
}

pub struct Render<'t, 'ttf_module, 'rwops> {
	pub graph: RenderGraphics<'t, 'ttf_module, 'rwops, i16, u32>,

	pub hot: WidgetId,
	pub active: WidgetId,
	//pub kbd: WidgetId,
	pub last: WidgetId,

	pub next_rect: Rect<i16>,

	pub key: Option<Key>,
	mouse: (bool, Point<i16>),

	cursors: Cursors,

	win: widgets::Root<i16, u32>,
}

impl<'t, 'ttf, 'rwops> Render<'t, 'ttf, 'rwops> {
	pub fn new(ctx: Canvas<Window>, creator: &'t TextureCreator<WindowContext>, font: Font<'ttf, 'rwops>) -> Self {
		Self {
			graph: RenderGraphics::new(ctx, creator, font),
			last: 0, hot: 0, active: 0, // kbd: 0,
			key: None,
			next_rect: Rect::with_coords(0, 0, 0, 0),
			mouse: (false, Point::new(0, 0)),
			win: widgets::example(),
			cursors: create_cursors(),
		}
	}

	pub fn mouse(&mut self, event: Mouse<i16>) {
		match event {
			Mouse::Move(p) => self.mouse.1 = p,
			Mouse::Press(p) => self.mouse = (true, p),
			Mouse::Release(p) => self.mouse = (true, p),
		}
		self.win.event(event);
	}

	pub fn prepare(&mut self, bg: u32) {
		self.hot = 0;
		self.graph.channel = 0;

		let bg = color!(bg);

		self.graph.ctx.set_viewport(None);
		self.graph.ctx.set_clip_rect(None);
		self.graph.ctx.set_draw_color(bg);
		self.graph.ctx.clear();
	}

	pub fn finish(&mut self) -> bool {
		self.win.redraw.set(true);
		self.win.paint(&mut self.graph);

		let last_active = self.active;
		//let last_kbd = self.kbd;

		if !self.mouse.0 {
			self.active = 0;
		} else if self.active == 0 {
			self.active = WidgetId::max_value();
		}

		// If no widget grabbed tab, clear focus
		/*if self.key == Some(Key::NextWidget) {
			self.kbd = 0;
		}*/

		// Clear the entered key
		self.key = None;

		let cur = if self.hot != 0 { SystemCursor::Hand } else { SystemCursor::Crosshair };
		self.cursors[&cur].set();

		self.graph.run_commands();
		self.graph.ctx.present();

		last_active != self.active //|| last_kbd != self.kbd
	}

}
impl<'t, 'ttf, 'rwops> Graphics<i16, u32> for Render<'t, 'ttf, 'rwops> {
	type RenderTarget = SdlCanvas;
	fn canvas<F: FnMut(&mut Self::RenderTarget, u32, u32)>(&mut self, id: usize, f: F) { self.graph.canvas(id, f) }
	fn command(&mut self, cmd: Command<i16, u32>) { self.graph.command(cmd) }
	fn text_size(&mut self, s: &str) -> (u32, u32) { self.graph.text_size(s) }
	fn image_size(&mut self, id: usize) -> (u32, u32) { self.graph.image_size(id) }
	fn channel(&mut self, ch: usize) { self.graph.channel(ch) }
	fn create_texture<T: Into<Option<usize>>>(&mut self, id: T, w: u32, h: u32) -> usize {
		self.graph.create_texture(id, w, h)
	}
}

impl<'t, 'ttf, 'rwops> Immediate for Render<'t, 'ttf, 'rwops> {
	fn bounds(&self) -> Rect<i16> {
		let size = self.graph.ctx.window().size();
		Rect::with_size(0, 0, size.0 as i16, size.1 as i16)
	}

	fn widget_rect(&self) -> Rect<i16> {
		self.next_rect
	}
	fn widget(&mut self, id: WidgetId) -> Rect<i16> {
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