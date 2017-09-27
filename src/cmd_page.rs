use undo::Command;
use std::error::Error;
use sprite::*;
use std::mem::swap;

pub struct PageCmd {
	page: Page,
	frame: usize,
	layer: usize,
}

impl PageCmd {
	pub fn new(frame: usize, layer: usize, page: Page) -> Self {
		Self { frame, layer, page }
	}
	fn run(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		self.page.swap(image.page_mut(self.frame, self.layer));
		Ok(())
	}
}

impl Command<Sprite> for PageCmd {
	fn redo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
	fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
}

pub enum ChangeColor {
	Foreground(u8),
	Background(u8),
}

impl ChangeColor {
	pub fn foreground(color: u8) -> Self {
		ChangeColor::Foreground(color)
	}
	fn run(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		match self {
			&mut ChangeColor::Foreground(ref mut c) => swap(c, &mut image.fg),
			&mut ChangeColor::Background(ref mut c) => swap(c, &mut image.bg),
		}
		Ok(())
	}
}

impl Command<Sprite> for ChangeColor {
	fn redo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
	fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
}