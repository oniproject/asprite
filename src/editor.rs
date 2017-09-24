use common::*;
use tool::*;

use cmd_page::*;
use sprite::*;

use undo::record::Record;

pub struct Editor<'a> {
	pub image: Record<'a, Sprite>,
	pub canvas: Page,

	pub frame: usize,
	pub layer: usize,

	pub pos: Point<i16>,

	pub m: Point<i16>,
	pub mouse: Point<i16>,
	pub zoom: i16,
	pub fg: u8,
	pub bg: u8,

	pub redraw: bool,
}

impl<'a> Editor<'a> {
	pub fn new(zoom: i16, pos: Point<i16>, image: Sprite) -> Self {
		Editor {
			zoom, pos,
			canvas: Page::new(image.width, image.height),
			image: Record::new(image),
			frame: 0, layer: 0,
			bg: 0,
			fg: 1,
			mouse: Point::new(-100, -100),
			m: Point::new(-100, -100),
			redraw: true,
		}
	}

	pub fn zoom(&mut self, y: i16) {
		let last = self.zoom;
		self.zoom += y;
		if self.zoom < 1 { self.zoom = 1 }
		if self.zoom > 16 { self.zoom = 16 }
		let diff = last - self.zoom;

		self.pos.x += self.size().x * diff / 2;
		self.pos.y += self.size().y * diff / 2;
	}

	pub fn size(&self) -> Point<i16> {
		Point::new(self.canvas.width as i16, self.canvas.height as i16)
	}

	pub fn set_mouse(&mut self, p: Point<i16>) -> Point<i16> {
		self.mouse = Point::from_coordinates((p - self.pos) / self.zoom);
		self.mouse
	}

	pub fn redo(&mut self) {
		self.image.redo();
		self.sync();
	}

	pub fn undo(&mut self) {
		self.image.undo();
		self.sync();
	}
}

impl<'a> Context for Editor<'a> {
	fn sync(&mut self) {
		self.canvas.copy_from(&self.image.as_receiver().page(0, 0));
		self.redraw = true;
	}

	fn start(&mut self) -> u8 {
		self.sync();
		self.fg
	}
	fn commit(&mut self) {
		let page = self.canvas.clone();
		let _ = self.image.push(PageCmd::new(0, 0, page));
		self.sync();
	}
	fn rollback(&mut self) {
		self.sync();
	}

	fn brush(&mut self, p: Point<i16>, color: u8) {
		let (w, h) = (self.canvas.width as i16, self.canvas.height as i16);
		let x = p.x >= 0 && p.x < w;
		let y = p.y >= 0 && p.y < h;
		if x && y {
			self.redraw = true;
			let idx = p.x + p.y * w;
			self.canvas.page[idx as usize] = color;
		}
	}

	fn pixel(&mut self, p: Point<i16>, color: u8) {
		let (w, h) = (self.canvas.width as i16, self.canvas.height as i16);
		let x = p.x >= 0 && p.x < w;
		let y = p.y >= 0 && p.y < h;
		if x && y {
			self.redraw = true;
			let idx = p.x + p.y * w;
			self.canvas.page[idx as usize] = color;
		}
	}
}