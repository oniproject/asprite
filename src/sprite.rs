use std::mem;
use common::*;

pub struct Sprite {
	pub data: Vec<Vec<Page>>,
	pub palette: Palette<u32>,
	pub width: usize,
	pub height: usize,
}

impl Sprite {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			data: vec![vec![Page::new(width, height)]],
			palette: Palette::new(0, None),
			width, height,
		}
	}
	pub fn page(&self, frame: usize, layer: usize) -> &Page {
		&self.data[frame][layer]
	}
	pub fn page_mut(&mut self, frame: usize, layer: usize) -> &mut Page {
		&mut self.data[frame][layer]
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
		self.page.copy_from_slice(&other.page);
	}
	pub fn swap(&mut self, other: &mut Page) {
		for (a, b) in self.page.iter_mut().zip(other.page.iter_mut()) {
			mem::swap(a, b);
		}
	}
}