use common::*;
use tool::*;

use cmd::*;
use sprite::*;
use ui;
use ui::*;

use sdl2;
use sdl2::pixels::Color;
use sdl2::gfx::primitives::DrawRenderer;


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
		let image = self.editor.image();
		image.palette[image.color]
	}

	pub fn draw(&mut self, render: &mut ui::Render<sdl2::video::Window>) {
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
				self.editor.redraw = false;
				self.editor.draw_pages(|page, stride, palette| {
					for (idx, c) in page.iter().enumerate() {
						let x = idx % stride;
						let y = idx / stride;
						canvas.pixel(x as i16, y as i16, palette[*c].to_be()).unwrap();
					}
				});
			}
			Layer::Preview => {
				canvas.set_draw_color(Color::RGBA(0x00, 0x00, 0x00, 0x00));
				canvas.clear();
				let image = self.editor.image();

				// freehand preview
				let color = self.freehand.color;
				for &(p, active) in &self.freehand.pts {
					let c = if active {
						image.palette[color].to_be()
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
