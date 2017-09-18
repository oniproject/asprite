use undo::Command;
use std::error::Error;
use sprite::*;

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