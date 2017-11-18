mod canvas;
mod sprite;
mod cmd_page;
pub use self::cmd_page::*;
pub use self::sprite::*;

use common::*;
use tool::*;
pub use self::canvas::Canvas;

use undo::record::Record;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub type ImageCell<'a> = Rc<RefCell<Record<'a, Sprite>>>;

pub struct Editor<'a> {
	pub image: ImageCell<'a>,
	pub redraw: Option<Rect<i32>>,

	pub brush_rect: Rect<i16>,
	pub brush_offset: Vector<i16>,

	canvas: Frame,
	rect: Rect<i32>,
	start: usize,
	stride: usize,
}

impl<'a> Editor<'a> {
	pub fn new(image: ImageCell<'a>) -> Self {
		let (w, h) = {
			let m = image.borrow();
			let m = m.as_receiver();
			(m.width, m.height)
		};
		let rect = Rect::with_size(0, 0, w as i32, h as i32);
		Self {
			image,
			rect,

			canvas: Frame::new(w, h),
			start: 0,
			stride: w,

			redraw: Some(rect),
			brush_rect: Rect::with_size(0, 0, 3, 3),
			brush_offset: Vector::new(-1, -1),
		}
	}

	pub fn sprite(&self) -> Ref<Record<'a, Sprite>> {
		self.image.borrow()
	}

	pub fn size(&self) -> Point<i32> {
		Point::new(self.rect.dx() as i32, self.rect.dy() as i32)
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

impl<'a> Canvas<u8, i32> for Editor<'a> {
	#[inline(always)]
	unsafe fn pixel_unchecked(&mut self, x: i32, y: i32, color: u8) {
		let x = x as usize;
		let y = y as usize;
		let idx = self.start + x + y * self.stride;
		*self.canvas.page.get_unchecked_mut(idx) = color;
	}

	#[inline(always)]
	fn update(&mut self, r: Rect<i32>) {
		self.redraw = match self.redraw {
			Some(r) => r.union(r),
			None => Some(r),
		};
	}

	#[inline(always)]
	fn intersect(&self, r: Rect<i32>) -> Option<Rect<i32>> {
		self.rect.intersect(r)
	}

	#[inline(always)]
	fn bounds(&self) -> Rect<i32> {
		self.rect
	}

	#[inline(always)]
	fn at(&self, x: i32, y: i32) -> Option<u8> {
		if self.rect.contains_xy(x, y) {
			let x = x as usize;
			let y = y as usize;
			let idx = self.start + x + y * self.stride;
			unsafe {
				Some(*self.canvas.page.get_unchecked(idx))
			}
		} else {
			None
		}
	}
}

impl<'a> Context<i32, u8> for Editor<'a> {
	fn sync(&mut self) {
		let m = self.image.borrow();
		let m = m.as_receiver();
		let (layer, frame) = {
			(m.layer.get(), m.frame.get())
		};
		self.canvas.copy_from(&m.page(layer, frame));
		self.redraw = Some(self.rect);
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

	fn paint_brush(&mut self, p: Point<i32>, color: u8) {
		let brush = [
			true, false, true,
			false, true, false,
			true, false, true,
		];
		let r = Rect::with_size(p.x, p.y, 3, 3)
			.xy(-1, -1);

		self.mask(r, &brush, color);
		self.update(r);
	}
}