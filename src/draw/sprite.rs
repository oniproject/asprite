use std::cell::Cell;

use super::{Frame, Layer, Palette};

pub struct Sprite {
    pub name: String,
    pub data: Vec<Layer>,
    pub palette: Palette<u32>,
    pub width: usize,
    pub height: usize,

    pub frame: Cell<usize>,
    pub layer: Cell<usize>,

    pub color: Cell<u8>,
}

impl Sprite {
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        Self {
            name: name.to_string(),
            data: Vec::new(),
            palette: Palette::new(0, None),
            width, height,
            frame: Cell::new(0),
            layer: Cell::new(0),
            color: Cell::new(1),
        }
    }

    pub fn is_lock(&self) -> bool {
        self.data[self.layer.get()].lock.get()
    }

    pub fn page(&self, layer: usize, frame: usize) -> &Frame {
        self.data[layer].get(frame)
    }
    pub fn page_mut(&mut self, layer: usize, frame: usize) -> &mut Frame {
        self.data[layer].get_mut(frame)
    }

    pub fn add_layer(&mut self, name: &str) {
        let mut layer = Layer::new(name);
        let page = Frame::new(self.width, self.height);
        layer.push(page);
        self.data.push(layer);
    }

    pub fn add_layer_page(&mut self, name: &str, page: Frame) {
        let mut layer = Layer::new(name);
        layer.push(page);
        self.data.push(layer);
    }
}
