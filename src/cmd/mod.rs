mod cmd_page;
pub use self::cmd_page::*;

use common::*;
use tool::*;
use sprite::*;

use undo::record::Record;

pub struct Editor<'a> {
	pub image: Record<'a, Sprite>,
	pub canvas: Page,

	pub redraw: bool,
}

impl<'a> Editor<'a> {
	pub fn new(image: Sprite) -> Self {
		Editor {
			canvas: Page::new(image.width, image.height).to_owned(),
			image: Record::new(image),
			redraw: true,
		}
	}

	pub fn draw_pages<F: FnMut(&Page, &Palette<u32>)>(&self, mut f: F) {
		let image = self.image.as_receiver();
		let current_layer = image.layer.get();
		let current_frame = image.frame.get();
		for (layer_id, layer) in image.data.iter().enumerate() {
			if !layer.visible.get() {
				continue;
			}
			for (frame_id, _) in layer.frames.iter().enumerate() {
				let is_canvas = layer_id == current_layer && frame_id == current_frame;
				let page = if is_canvas {
					Some(&self.canvas)
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
		let w = self.canvas.width as i16;
		let h = self.canvas.height as i16;
		let r = Rect::with_size(0, 0, w, h);
		let pp = Rect::with_size(p.x, p.y, 1, 1);
		if let Some(_r) = r.intersect(pp) {
			self.redraw = true;
			let idx = p.x + p.y * w;
			self.canvas.page[idx as usize] = color;
		}
	}

	fn paint_pixel(&mut self, p: Point<i16>, color: u8) {
		let (w, h) = (self.canvas.width as i16, self.canvas.height as i16);
		let x = p.x >= 0 && p.x < w;
		let y = p.y >= 0 && p.y < h;
		if x && y {
			self.redraw = true;
			let idx = p.x + p.y * w;
			self.canvas.page[idx as usize] = color;
		}
	}
}

impl<'a> Context<i16, u8> for Editor<'a> {
	fn sync(&mut self) {
		let m = self.image.as_receiver();
		let (layer, frame) = {
			(m.layer.get(), m.frame.get())
		};
		self.canvas.copy_from(&m.page(layer, frame));
		self.redraw = true;
	}

	fn start(&mut self) -> u8 {
		self.sync();
		self.image.as_receiver().color.get()
	}
	fn commit(&mut self) {
		let page = self.canvas.clone();
		let (layer, frame) = {
			let m = self.image.as_receiver();
			(m.layer.get(), m.frame.get())
		};
		let _ = self.image.push(DrawCommand::new(layer, frame, page.clone()));
		self.sync();
	}
	fn rollback(&mut self) {
		self.sync();
	}

	fn change_color(&mut self, color: u8) {
		let _ = self.image.as_receiver().color.set(color);
	}
}