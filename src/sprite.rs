use std::mem;
use common::*;

pub struct Layer {
	pub frames: Vec<Page>,
	pub name: String,
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
		Self {
			data: vec![Layer {
				frames: vec![Page::new(width, height)],
				name: "Layer".to_string(),
			}],
			palette: Palette::new(0, None),
			width, height,
			fg: 0,
			bg: 1,
		}
	}
	pub fn page(&self, frame: usize, layer: usize) -> &Page {
		&self.data[layer].frames[frame]
	}
	pub fn page_mut(&mut self, frame: usize, layer: usize) -> &mut Page {
		&mut self.data[layer].frames[frame]
	}

	pub fn add_frame(&mut self) {}
	pub fn add_layer(&mut self, name: &str) {
		unimplemented!();
		self.data.push(Layer {
			frames: vec![Page::new(self.width, self.height)],
			name: name.to_string(),
		});
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
		assert_eq!(self.width, other.width);
		assert_eq!(self.height, other.height);
		self.page.copy_from_slice(&other.page);
	}
	pub fn swap(&mut self, other: &mut Page) {
		assert_eq!(self.width, other.width);
		assert_eq!(self.height, other.height);
		for (a, b) in self.page.iter_mut().zip(other.page.iter_mut()) {
			mem::swap(a, b);
		}
	}
}