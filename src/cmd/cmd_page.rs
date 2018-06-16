use undo::Command;
use std::error::Error;
use std::mem::swap;

use super::{Frame, Sprite};

#[derive(Debug)]
pub struct DrawCommand {
    page: Frame,
    frame: usize,
    layer: usize,
}

impl DrawCommand {
    pub fn new(frame: usize, layer: usize, page: Frame) -> Self {
        Self { frame, layer, page }
    }
    fn run(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> {
        swap(&mut self.page, image.page_mut(self.frame, self.layer));
        Ok(())
    }
}

impl Command<Sprite> for DrawCommand {
    fn apply(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
    fn undo(&mut self, image: &mut Sprite) -> Result<(), Box<Error>> { self.run(image) }
}
