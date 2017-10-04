use common::*;
use tool::*;

use cmd::*;
use sprite::*;
use ui;
use ui::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentTool {
	Freehand,
	Bucket,
	EyeDropper,
	Primitive(PrimitiveMode),
}

#[derive(PartialEq)]
pub enum Layer {
	Sprite,
	Preview,
}

pub struct Tools<'a> {
	pub current: CurrentTool,
	pub editor: Editor<'a>,

	pub freehand: Freehand<i16, u8>,
	pub prim: Primitive<i16, u8>,
	pub bucket: Bucket<i16, u8>,
	pub dropper: EyeDropper<i16, u8>,

	pub pos: Point<i16>,
	pub grid: Option<Point<i16>>,

	pub drag: bool,

	pub m: Point<i16>,
	pub mouse: Point<i16>,
	pub zoom: i16,
}

impl<'a> Tools<'a> {
	pub fn new(zoom: i16, pos: Point<i16>, sprite: Sprite) -> Self {
		Self {
			zoom, pos,
			mouse: Point::new(-100, -100),
			m: Point::new(-100, -100),
			drag: false,

			grid: Some(Point::new(16, 16)),

			current: CurrentTool::Freehand,
			prim: Primitive::new(),
			bucket: Bucket::new(),
			freehand: Freehand::new(),
			dropper: EyeDropper::new(),
			editor: Editor::new(sprite),
		}
	}

	pub fn input(&mut self, ev: Input<i16>) {
		if self.editor.image.as_receiver().is_lock() {
			return
		}
		match self.current {
			CurrentTool::Freehand => self.freehand.run(ev, &mut self.editor),
			CurrentTool::Bucket => self.bucket.run(ev, &mut self.editor),
			CurrentTool::EyeDropper => self.dropper.run(ev, &mut self.editor),
			CurrentTool::Primitive(mode) => {
				self.prim.mode = mode;
				self.prim.run(ev, &mut self.editor)
			}
		}
	}

	pub fn mouse_press(&mut self, p: Point<i16>) {
		let p = self.set_mouse(p);
		if p.x >= 0 && p.y >= 0 {
			self.input(Input::Press(p));
		}
	}

	pub fn mouse_release(&mut self, p: Point<i16>) {
		let p = self.set_mouse(p);
		if p.x >= 0 && p.y >= 0 {
			self.input(Input::Release(p));
		}
	}

	pub fn mouse_move(&mut self, p: Point<i16>, v: Vector<i16>) {
		let p = self.set_mouse(p);
		if self.drag {
			self.pos += v;
		} else {
			self.input(Input::Move(p));
		}
	}

	pub fn zoom_from_center(&mut self, y: i16) {
		let p = self.size();
		self.zoom(y, |diff| p * diff / 2);
	}

	pub fn zoom_from_mouse(&mut self, y: i16) {
		let p = self.mouse;
		self.zoom(y, |diff| p * diff);
	}

	fn zoom<F: FnOnce(i16) -> Point<i16>>(&mut self, y: i16, f: F) {
		let last = self.zoom;
		self.zoom += y;
		if self.zoom < 1 { self.zoom = 1 }
		if self.zoom > 16 { self.zoom = 16 }
		let diff = last - self.zoom;

		let p = f(diff);

		self.pos.x += p.x;
		self.pos.y += p.y;
	}

	pub fn set_mouse(&mut self, p: Point<i16>) -> Point<i16> {
		self.mouse = Point::from_coordinates((p - self.pos) / self.zoom);
		self.mouse
	}

	pub fn size(&self) -> Point<i16> {
		Point::new(self.editor.canvas.width as i16, self.editor.canvas.height as i16)
	}

	pub fn redo(&mut self) {
		self.editor.image.redo();
		self.editor.sync();
	}

	pub fn undo(&mut self) {
		self.editor.image.undo();
		self.editor.sync();
	}

	pub fn color(&self) -> u32 {
		let m = self.editor.image.as_receiver();
		m.palette[m.color.get()]
	}

	pub fn pal(&self, color: u8) -> u32 {
		let m = self.editor.image.as_receiver();
		m.palette[color]
	}

	pub fn color_index(&self) -> u8 {
		let m = self.editor.image.as_receiver();
		m.color.get()
	}

	pub fn draw(&mut self, render: &mut ui::Render) {
		let red = 0xFF4136_FFu32;

		let textures = {
			// borrow checker awesome
			let textures = render.textures.iter_mut().filter_map(|(key, value)| {
				match *key {
					EDITOR_SPRITE_ID => Some((&mut value.0, Layer::Sprite)),
					EDITOR_PREVIEW_ID => Some((&mut value.0, Layer::Preview)),
					_ => None
				}
			// FIXME: maybe use mem::uninitialized? or some other?
			}).fold((None, None), |acc, v| {
				match v.1 {
					Layer::Sprite => { (Some(v), acc.1) },
					Layer::Preview => { (acc.0, Some(v)) },
				}
			});
			[textures.0.unwrap(), textures.1.unwrap()]
		};

		render.ctx.with_multiple_texture_canvas(textures.iter(), |canvas, layer| {
			match *layer {
			Layer::Sprite => if self.editor.redraw {
				let clear_color = color!(TRANSPARENT);
				//let clear_color = color!(self.pal(0));
				canvas.set_draw_color(clear_color);
				canvas.clear();
				self.editor.redraw = false;
				self.editor.draw_pages(|page, palette| {
					let stride = page.width;
					let transparent = page.transparent;
					for (idx, &c) in page.page.iter().enumerate() {
						let x = idx % stride;
						let y = idx / stride;
						if Some(c) != transparent {
							canvas.pixel(x as i16, y as i16, palette[c].to_be()).unwrap();
						}
					}
				});
			}
			Layer::Preview => {
				canvas.set_draw_color(color!(TRANSPARENT));
				canvas.clear();

				// freehand preview
				let color = self.freehand.color;
				for &(p, active) in &self.freehand.pts {
					let c = if active {
						self.pal(color).to_be()
					} else {
						red
					};
					canvas.pixel(p.x, p.y, c).unwrap();
				}

				// preview brush
				canvas.pixel(self.mouse.x, self.mouse.y, self.color().to_be()).unwrap();
			}
			}
		}).unwrap();

		{ // display image and preview
			let size = self.size();
			let zoom = self.zoom;
			let size = size * zoom;
			let pos = self.pos;
			let dst = rect!(pos.x, pos.y, size.x, size.y);
			for t in textures.iter() {
				render.ctx.copy(t.0, None, dst).unwrap();
			}
		}

		{ // grid
			let rr = GRID_COLOR.to_be();

			let (ox, oy) = (self.pos.x, self.pos.y);
			let size = self.size();
			let zoom = self.zoom;

			let (x1, x2) = (ox, ox + size.x * zoom);
			let (y1, y2) = (oy, oy + size.y * zoom);

			if let Some(grid) = self.grid {
				let ex = size.x / grid.x;
				let ey = size.y / grid.y;
				let ix = (size.x % grid.x != 0) as i16;
				let iy = (size.y % grid.y != 0) as i16;

				for x in 1..ex + ix {
					let x = ox - 1 + (x * grid.x) * zoom;
					render.ctx.vline(x, y1, y2, rr).unwrap();
				}

				for y in 1..ey + iy {
					let y = oy - 1 + (y * grid.y) * zoom;
					render.ctx.hline(x1, x2, y, rr).unwrap();
				}
			}

			let gg = CORNER_COLOR.to_be();

			// canvas border
			render.ctx.hline(x1-1, x2, y1-1, gg).unwrap();
			render.ctx.hline(x1-1, x2, y2+0, gg).unwrap();
			render.ctx.vline(x1-1, y1-1, y2, gg).unwrap();
			render.ctx.vline(x2+0, y1-1, y2, gg).unwrap();
		}
	}
}
