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

	pub size: Point<i16>,
	pub pos: Point<i16>,

	pub m: Point<i16>,
	pub mouse: Point<i16>,
	pub zoom: i16,
	pub fg: u8,
	pub bg: u8,
}

impl<'a> Editor<'a> {
	pub fn new(zoom: i16, pos: Point<i16>, size: Point<i16>) -> Self {
		let len = (size.x * size.y) as usize;
		Editor {
			zoom, pos, size,
			image: Record::new(Sprite{ data: vec![vec![Page::new(len)]] }),
			canvas: Page::new(len),
			frame: 0, layer: 0,
			bg: 0,
			fg: 1,
			mouse: Point::new(-100, -100),
			m: Point::new(-100, -100),
		}
	}

	pub fn set_mouse(&mut self, p: Point<i16>) -> Point<i16> {
		self.mouse = Point::from_coordinates((p - self.pos) / self.zoom);
		self.mouse
	}

	fn sync(&mut self) {
		self.canvas.copy_from(&self.image.as_receiver().page(0, 0));
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
		let x = p.x >= 0 && p.x < self.size.x;
		let y = p.y >= 0 && p.y < self.size.y;
		if x && y {
			let idx = p.x + p.y * self.size.x;
			self.canvas.page[idx as usize] = color;
		}
	}
}