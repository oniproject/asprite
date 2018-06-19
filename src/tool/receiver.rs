use std::cell::Cell;
use draw::{Frame, Palette};

use math::{Rect, Vector2, Point2};

pub struct Receiver {
    pub data: Vec<Layer>,

    pub name: String,
    pub palette: Palette<u32>,
    pub width: usize,
    pub height: usize,

    pub frame: Cell<usize>,
    pub layer: Cell<usize>,

    pub color: Cell<u8>,

    pub zoom: i32,
    pub pos: Point2<i32>,

    pub created: bool,
    pub redraw: Option<Rect<i32>>,
}

impl Receiver {
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        let rect = Rect::from_coords_and_size(0, 0, width as i32, height as i32);
        Self {
            name: name.to_string(),
            data: Vec::new(),
            palette: Palette::new(0, None),
            width, height,
            frame: Cell::new(0),
            layer: Cell::new(0),
            color: Cell::new(1),

            zoom: 1,
            pos: Point2::new(0, 0),

            created: false,
            redraw: Some(rect),
        }
    }

    pub fn take_update(&mut self) -> Option<Rect<i32>> {
        self.redraw.take()
    }

    pub fn update_all(&mut self) {
        let w = self.width as i32;
        let h = self.height as i32;
        self.redraw = Some(Rect::from_coords_and_size(0, 0, w, h));
    }

    pub fn update(&mut self, r: Rect<i32>) {
        self.redraw = match self.redraw {
            Some(r) => r.union(r),
            None => Some(r),
        };
    }

    pub fn rect(&self) -> Rect<i32> {
        let dim = Vector2::new(self.width as i32, self.height as i32);
        Rect::from_min_dim(self.pos, dim)
    }

    pub fn zoom<F: FnOnce(i32) -> Vector2<i32>>(&mut self, y: i32, f: F) {
        let last = self.zoom;
        self.zoom += y;
        if self.zoom < 1 { self.zoom = 1 }
        if self.zoom > 16 { self.zoom = 16 }
        let diff = last - self.zoom;

        let p = f(diff);

        self.pos.x += p.x;
        self.pos.y += p.y;
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
