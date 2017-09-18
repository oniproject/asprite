use std::mem;

pub struct Sprite {
	pub data: Vec<Vec<Page>>,
}

impl Sprite {
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
}
impl Page {
	pub fn new(len: usize) -> Page {
		Self {
			page: vec![0; len]
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