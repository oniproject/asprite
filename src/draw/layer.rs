use std::cell::Cell;
use super::Frame;

pub struct Layer {
    pub frames: Vec<Frame>,
    pub name: String,
    pub visible: Cell<bool>,
    pub lock: Cell<bool>,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            frames: Vec::new(),
            name: name.to_string(),
            visible: Cell::new(true),
            lock: Cell::new(false),
        }
    }

    pub fn get(&self, idx: usize) -> &Frame {
        &self.frames[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Frame {
        &mut self.frames[idx]
    }

    pub fn push(&mut self, page: Frame) {
        self.frames.push(page)
    }
    pub fn insert(&mut self, pos: usize, page: Frame) {
        self.frames.insert(pos, page)
    }
    pub fn remove(&mut self, pos: usize) -> Frame {
        self.frames.remove(pos)
    }
}
