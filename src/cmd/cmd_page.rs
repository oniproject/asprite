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

pub struct ChangeColor(pub u8);

impl ChangeColor {
	fn run(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		swap(&mut self.0, &mut image.color);
		Ok(())
	}
}

impl Command<Sprite> for ChangeColor {
	fn redo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
	fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }

	fn id(&self) -> Option<u32> { Some(0) }
}