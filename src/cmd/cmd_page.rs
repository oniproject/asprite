use undo::Command;
use std::error::Error;
use sprite::*;
use std::mem::swap;

pub struct DrawCommand {
	page: Page,
	frame: usize,
	layer: usize,
}

impl DrawCommand {
	pub fn new(frame: usize, layer: usize, page: Page) -> Self {
		DrawCommand { frame, layer, page }
	}
	fn run(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		swap(&mut self.page, image.page_mut(self.frame, self.layer));
		Ok(())
	}
}

impl Command<Sprite> for DrawCommand {
	fn redo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
	fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
}