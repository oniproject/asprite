mod cmd_page;
pub use self::cmd_page::*;

use common::*;
use tool::*;
use sprite::*;

use undo::record::Record;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub type ImageCell<'a> = Rc<RefCell<Record<'a, Sprite>>>;

pub struct Editor<'a> {
	pub image: ImageCell<'a>,
	canvas: Frame,
	pub redraw: bool,
	pub brush_rect: Rect<i16>,
	pub brush_offset: Vector<i16>,
}

impl<'a> Editor<'a> {
	pub fn new(image: ImageCell<'a>) -> Self {
		let (w, h) = {
			let m = image.borrow();
			let m = m.as_receiver();
			(m.width, m.height)
		};
		Self {
			image,
			canvas: Frame::new(w, h).to_owned(),
			redraw: true,
			brush_rect: Rect::with_size(0, 0, 3, 3),
			brush_offset: Vector::new(-1, -1),
		}
	}

	pub fn sprite(&self) -> Ref<Record<'a, Sprite>> {
		self.image.borrow()
	}

	pub fn size(&self) -> Point<i16> {
		Point::new(self.canvas.width as i16, self.canvas.height as i16)
	}

	pub fn redo(&mut self) {
		self.image.borrow_mut().redo();
		self.sync();
	}

	pub fn undo(&mut self) {
		self.image.borrow_mut().undo();
		self.sync();
	}

	pub fn draw_pages<F: FnMut(&Frame, &Palette<u32>)>(&self, mut f: F) {
		let image = self.sprite();
		let image = image.as_receiver();
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
		let p = p + self.brush_offset;
		let pp = self.brush_rect.xy(p.x, p.y);

		let pix = self.canvas.page.as_mut_ptr();
		if let Some(r) = r.intersect(pp) {
			self.redraw = true;
			for y in r.min.y..r.max.y {
				for x in r.min.x..r.max.x {
					let x = x as isize;
					let y = y as isize;
					let w = w as isize;
					let idx = x + y * w;
					unsafe { *pix.offset(idx) = color; }
				}
			}
		}
	}

	fn paint_pixel(&mut self, p: Point<i16>, color: u8) {
		let (w, h) = (self.canvas.width as i16, self.canvas.height as i16);
		let pix = self.canvas.page.as_mut_ptr();
		if p.x >= 0 && p.x < w && p.y >= 0 && p.y < h {
			self.redraw = true;
			let x = p.x as isize;
			let y = p.y as isize;
			let w = w as isize;
			let idx = x + y * w;
			unsafe { *pix.offset(idx) = color; }
		}
	}
}

impl<'a> Context<i16, u8> for Editor<'a> {
	fn sync(&mut self) {
		let m = self.image.borrow();
		let m = m.as_receiver();
		let (layer, frame) = {
			(m.layer.get(), m.frame.get())
		};
		self.canvas.copy_from(&m.page(layer, frame));
		self.redraw = true;
	}

	fn start(&mut self) -> u8 {
		self.sync();
		self.sprite().as_receiver().color.get()
	}
	fn commit(&mut self) {
		let page = self.canvas.clone();
		let (layer, frame) = {
			let m = self.sprite();
			let m = m.as_receiver();
			(m.layer.get(), m.frame.get())
		};
		let _ = self.image.borrow_mut().push(DrawCommand::new(layer, frame, page.clone()));
		self.sync();
	}
	fn rollback(&mut self) {
		self.sync();
	}

	fn change_color(&mut self, color: u8) {
		let _ = self.sprite().as_receiver().color.set(color);
	}
}