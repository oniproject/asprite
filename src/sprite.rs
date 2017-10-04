#![allow(dead_code)]
use common::*;

pub struct Sprite {
	pub data: Vec<Layer>,
	pub palette: Palette<u32>,
	pub width: usize,
	pub height: usize,

	pub frame: usize,
	pub layer: usize,

	pub color: u8,
}

impl Sprite {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			data: Vec::new(),
			palette: Palette::new(0, None),
			width, height,
			frame: 0,
			layer: 0,
			color: 0,
		}
	}
	pub fn page(&self, layer: usize, frame: usize) -> &Page {
		self.data[layer].get(frame)
	}
	pub fn page_mut(&mut self, layer: usize, frame: usize) -> &mut Page {
		self.data[layer].get_mut(frame)
	}

	pub fn add_layer(&mut self, name: &str) {
		let mut layer = Layer::new(name);
		let page = Page::new(self.width, self.height);
		layer.push(page);
		self.data.push(layer);
	}
}

pub struct Layer {
	pub frames: Vec<Page>,
	pub name: String,
	pub visible: bool,
}

impl Layer {
	pub fn new(name: &str) -> Self {
		Self {
			frames: Vec::new(),
			name: name.to_string(),
			visible: true,
		}
	}

	pub fn get(&self, idx: usize) -> &Page {
		&self.frames[idx]
	}

	pub fn get_mut(&mut self, idx: usize) -> &mut Page {
		&mut self.frames[idx]
	}

	pub fn push(&mut self, page: Page) {
		self.frames.push(page)
	}
	pub fn insert(&mut self, pos: usize, page: Page) {
		self.frames.insert(pos, page)
	}
	pub fn remove(&mut self, pos: usize) -> Page {
		self.frames.remove(pos)
	}
}

#[derive(Clone)]
pub struct Page {
	pub page: Vec<u8>,
	pub transparent: Option<u8>,
	pub width: usize,
	pub height: usize,
}
impl Page {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			page: vec![0; width * height],
			transparent: Some(0),
			width, height,
		}
	}
	pub fn copy_from(&mut self, other: &Page) {
		self.width = other.width;
		self.height = other.height;
		self.transparent = other.transparent;
		self.page.resize(other.page.len(), 0);
		self.page.copy_from_slice(&other.page);
	}
}