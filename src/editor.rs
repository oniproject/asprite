use common::*;
use tool::*;

use cmd_page::*;
use sprite::*;

use undo::record::Record;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentTool {
	Freehand,
	Bucket,
	EyeDropper,
	Primitive(PrimitiveMode),
}

pub struct Tools<'a> {
	pub current: CurrentTool,
	pub editor: Editor<'a>,

	pub freehand: Freehand<i16, u8>,
	pub prim: Primitive<i16, u8>,
	pub bucket: Bucket<i16, u8>,
	pub dropper: EyeDropper<i16, u8>,
}

impl<'a> Tools<'a> {
	pub fn input(&mut self, ev: Input<i16>) {
		match self.current {
			CurrentTool::Freehand => {
				self.freehand.run(ev, &mut self.editor);
			}
			CurrentTool::Bucket => self.bucket.run(ev, &mut self.editor),
			CurrentTool::EyeDropper => self.dropper.run(ev, &mut self.editor),
			CurrentTool::Primitive(mode) => {
				self.prim.mode = mode;
				self.prim.run(ev, &mut self.editor)
			}
		}
	}
}

pub struct Editor<'a> {
	pub image: Record<'a, Sprite>,
	pub canvas: Cow<'a, Page>,

	pub frame: usize,
	pub layer: usize,

	pub pos: Point<i16>,

	pub m: Point<i16>,
	pub mouse: Point<i16>,
	pub zoom: i16,

	pub redraw: bool,
}

impl<'a> Editor<'a> {
	pub fn new(zoom: i16, pos: Point<i16>, image: Sprite) -> Self {
		Editor {
			zoom, pos,
			canvas: Cow::Owned(Page::new(image.width, image.height).to_owned()),
			image: Record::new(image),
			frame: 0, layer: 0,
			mouse: Point::new(-100, -100),
			m: Point::new(-100, -100),
			redraw: true,
		}
	}

	pub fn zoom_from_center(&mut self, y: i16) {
		let last = self.zoom;
		self.zoom += y;
		if self.zoom < 1 { self.zoom = 1 }
		if self.zoom > 16 { self.zoom = 16 }
		let diff = last - self.zoom;

		let size = self.size() * diff / 2;

		self.pos.x += size.x;
		self.pos.y += size.y;
	}

	pub fn zoom_from_mouse(&mut self, y: i16) {
		let last = self.zoom;
		self.zoom += y;
		if self.zoom < 1 { self.zoom = 1 }
		if self.zoom > 16 { self.zoom = 16 }
		let diff = last - self.zoom;

		//let size = self.size() * diff / 2;
		let size = self.mouse * diff;

		self.pos.x += size.x;
		self.pos.y += size.y;
	}

	pub fn size(&self) -> Point<i16> {
		Point::new(self.canvas.width as i16, self.canvas.height as i16)
	}

	pub fn set_mouse(&mut self, p: Point<i16>) -> Point<i16> {
		self.mouse = Point::from_coordinates((p - self.pos) / self.zoom);
		self.mouse
	}

	pub fn redo(&mut self) {
		self.image.redo();
		self.sync();
	}

	pub fn undo(&mut self) {
		self.image.undo();
		self.sync();
	}

	pub fn image(&self) -> &Sprite {
		self.image.as_receiver()
	}

	pub fn fg(&self) -> u32 {
		let image = self.image();
		image.palette[image.fg]
	}

	pub fn draw_pages<F: FnMut(&Vec<u8>, usize, &Palette<u32>)>(&self, mut f: F) {
		let image = self.image();
		for (layer_id, layer) in image.data.iter().enumerate() {
			for (frame_id, _) in layer.frames.iter().enumerate() {
				let is_canvas = layer_id == self.layer && frame_id == self.frame;
				let page = if is_canvas {
					Some(self.canvas.as_ref())
				} else {
					Some(image.page(layer_id, frame_id))
				};
				if let Some(page) = page {
					f(&page.page, page.width, &image.palette)
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
		self.canvas.to_mut().copy_from(&self.image.as_receiver().page(0, 0));
		self.redraw = true;
	}

	fn start(&mut self) -> u8 {
		self.sync();
		self.image().fg
	}
	fn commit(&mut self) {
		let page = self.canvas.clone();
		let _ = self.image.push(PageCmd::new(0, 0, page.into_owned()));
		self.sync();
	}
	fn rollback(&mut self) {
		self.sync();
	}

	fn change_foreground(&mut self, color: u8) {
		let _ = self.image.push(ChangeColor::Foreground(color));
	}

	fn change_background(&mut self, color: u8) {
		let _ = self.image.push(ChangeColor::Background(color));
	}
}