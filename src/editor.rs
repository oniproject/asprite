use common::*;
use tool::*;

pub type Page = Vec<u8>;

use redo::Command;
use redo::record::Record;
use std::mem;

pub struct Editor<'a> {
	pub image: Record<'a, Page, Cmd>,

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
			image: Record::new(vec![0u8; len]),
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

	pub fn redo(&mut self) {
		self.image.redo();
	}

	pub fn undo(&mut self) {
		self.image.undo();
	}
}

impl<'a> Context for Editor<'a> {
	fn start(&mut self) -> u8 {
		self.fg
	}
	fn commit(&mut self) {
		self.image.push(Cmd::finish());
	}
	fn rollback(&mut self) {
	}

	fn brush(&mut self, p: Point<i16>, color: u8) {
		let x = p.x >= 0 && p.x < self.size.x;
		let y = p.y >= 0 && p.y < self.size.y;
		if x && y {
			let idx = p.x + p.y * self.size.x;
			self.image.push(Cmd::next(idx as u32, color));
		}
	}
}

struct Paint(Vec<(u32, u8)>);
impl Paint {
	fn swap(&mut self, page: &mut Page) {
		for d in self.0.iter_mut().rev() {
			mem::swap(&mut page[d.0 as usize], &mut d.1);
		}
	}
	fn merge(&mut self, mut other: Self) {
		self.0.append(&mut other.0);
	}
}

pub struct Cmd {
	data: Paint,
	finish: bool,
}

impl Cmd {
	fn next(idx: u32, color: u8) -> Self {
		Self {
			data: Paint(vec![(idx, color)]),
			finish: false,
		}
	}
	fn finish() -> Self {
		Self {
			data: Paint(Vec::new()),
			finish: true,
		}
	}
}

impl Command<Page> for Cmd {
	type Err = ();

	fn redo(&mut self, page: &mut Page) -> Result<(), Self::Err> {
		self.data.swap(page);
		Ok(())
	}
	fn undo(&mut self, page: &mut Page) -> Result<(), Self::Err> {
		self.data.swap(page);
		Ok(())
	}

	fn merge(&mut self, cmd: Self) -> Result<(), Self> {
		if self.finish {
			Err(cmd)
		} else {
			if cmd.data.0.len() == 0 {
				self.finish = true;
			}
			self.data.merge(cmd.data);
			Ok(())
		}
	}
}