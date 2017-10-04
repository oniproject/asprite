mod cmd_page;
pub use self::cmd_page::*;

use common::*;
use tool::*;
use sprite::*;

use undo::record::Record;
use std::borrow::Cow;

pub struct Editor<'a> {
	pub image: Record<'a, Sprite>,
	pub canvas: Cow<'a, Page>,

	pub redraw: bool,
}

impl<'a> Editor<'a> {
	pub fn new(image: Sprite) -> Self {
		Editor {
			canvas: Cow::Owned(Page::new(image.width, image.height).to_owned()),
			image: Record::new(image),
			redraw: true,
		}
	}

	pub fn select_layer(&mut self, layer: usize) {
		let _ = self.image.push(ChangeLayer(layer));
		self.sync();
	}

	pub fn draw_pages<F: FnMut(&Page, &Palette<u32>)>(&self, mut f: F) {
		let image = self.image.as_receiver();
		let current_layer = image.layer;
		let current_frame = image.frame;
		for (layer_id, layer) in image.data.iter().enumerate() {
			if !layer.visible {
				continue;
			}
			for (frame_id, _) in layer.frames.iter().enumerate() {
				let is_canvas = layer_id == current_layer && frame_id == current_frame;
				let page = if is_canvas {
					Some(self.canvas.as_ref())
				} else {
					Some(image.page(layer_id, frame_id))
				};
				if let Some(page) = page {
					f(&page, &image.palette)
				}
			}
		}
	}
}

impl<'a> Image<i16, u8> for Editor<'a> {
	fn width(&self) -> i16 { self.canvas.width as i16 }
	fn height(&self) -> i16 { self.canvas.height as i16 }

	fn at(&self, x: i16, y: i16) -> Option<u8> {
		let (w, h) = (self.canvas.width as i16, self.canvas.height as i16);
		if x >= 0 && x < w && y >= 0 && y < h {
			let idx = x + y * w;
			Some(self.canvas.page[idx as usize])
		} else {
			None
		}
	}

	fn paint_brush(&mut self, p: Point<i16>, color: u8) {
		let (w, h) = (self.canvas.width as i16, self.canvas.height as i16);
		let x = p.x >= 0 && p.x < w;
		let y = p.y >= 0 && p.y < h;
		if x && y {
			self.redraw = true;
			let idx = p.x + p.y * w;
			self.canvas.to_mut().page[idx as usize] = color;
		}
	}

	fn paint_pixel(&mut self, p: Point<i16>, color: u8) {
		let (w, h) = (self.canvas.width as i16, self.canvas.height as i16);
		let x = p.x >= 0 && p.x < w;
		let y = p.y >= 0 && p.y < h;
		if x && y {
			self.redraw = true;
			let idx = p.x + p.y * w;
			self.canvas.to_mut().page[idx as usize] = color;
		}
	}
}

impl<'a> Context<i16, u8> for Editor<'a> {
	fn sync(&mut self) {
		let m = self.image.as_receiver();
		self.canvas.to_mut().copy_from(&m.page(m.layer, m.frame));
		self.redraw = true;
	}

	fn start(&mut self) -> u8 {
		self.sync();
		self.image.as_receiver().color
	}
	fn commit(&mut self) {
		let page = self.canvas.clone();
		let (layer, frame) = {
			let m = self.image.as_receiver();
			(m.layer, m.frame)
		};
		let _ = self.image.push(DrawCommand::new(layer, frame, page.into_owned()));
		self.sync();
	}
	fn rollback(&mut self) {
		self.sync();
	}

	fn change_color(&mut self, color: u8) {
		let _ = self.image.push(ChangeColor(color));
	}
}