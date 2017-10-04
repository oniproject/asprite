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

pub struct ChangeFrame(pub usize);

impl ChangeFrame {
	fn run(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		swap(&mut self.0, &mut image.frame);
		Ok(())
	}
}

impl Command<Sprite> for ChangeFrame {
	fn redo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
	fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }

	fn id(&self) -> Option<u32> { Some(1) }
}

pub struct ChangeLayer(pub usize);

impl ChangeLayer {
	fn run(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		swap(&mut self.0, &mut image.layer);
		Ok(())
	}
}

impl Command<Sprite> for ChangeLayer {
	fn redo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
	fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }

	fn id(&self) -> Option<u32> { Some(1) }
}

pub struct LayerVisible(pub usize, pub bool);

impl Command<Sprite> for LayerVisible {
	fn redo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		image.data[self.0].visible = self.1;
		Ok(())
	}
	fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
		image.data[self.0].visible = !self.1;
		Ok(())
	}
}