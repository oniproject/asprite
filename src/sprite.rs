#![allow(dead_code)]
use common::*;

use std::mem::swap;

pub struct Layer {
	pub frames: Vec<Page>,
	pub name: String,
}

impl Layer {
	pub fn new(name: &str) -> Self {
		Self {
			frames: Vec::new(),
			name: name.to_string(),
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

pub struct Sprite {
	pub data: Vec<Layer>,
	pub palette: Palette<u32>,
	pub width: usize,
	pub height: usize,

	pub fg: u8,
	pub bg: u8,
}

impl Sprite {
	pub fn new(width: usize, height: usize) -> Self {
		let mut layer = Layer::new("Layer");
		layer.push(Page::new(width, height));
		Self {
			data: vec![layer],
			palette: Palette::new(0, None),
			width, height,
			fg: 0,
			bg: 1,
		}
	}
	pub fn page(&self, frame: usize, layer: usize) -> &Page {
		self.data[layer].get(frame)
	}
	pub fn page_mut(&mut self, frame: usize, layer: usize) -> &mut Page {
		self.data[layer].get_mut(frame)
	}

	pub fn add_frame(&mut self) {}
	pub fn add_layer(&mut self, name: &str) {
		let mut layer = Layer::new(name);
		layer.push(Page::new(self.width, self.height));
		self.data.push(layer);
		unimplemented!();
	}
}

#[derive(Clone)]
pub struct Page {
	pub page: Vec<u8>,
	pub width: usize,
	pub height: usize,
}
impl Page {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			page: vec![0; width * height],
			width, height,
		}
	}
	pub fn copy_from(&mut self, other: &Page) {
		self.width = other.width;
		self.height = other.height;
		self.page.resize(other.page.len(), 0);
		self.page.copy_from_slice(&other.page);
	}
	pub fn swap(&mut self, other: &mut Page) {
		swap(self, other);
	}
}