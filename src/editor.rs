use common::*;
use tool::*;

use cmd::*;
use ui;
use ui::*;

use grid::*;

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

	pub pos: Point<i16>,
	pub grid: Grid,

	pub drag: bool,

	pub m: Point<i16>,
	pub mouse: Point<i16>,
	pub zoom: i16,

	pub created: bool,
}

impl<'a> Tools<'a> {
	pub fn new(zoom: i16, pos: Point<i16>, sprite: ImageCell<'a>) -> Self {
		Self {
			zoom, pos,
			mouse: Point::new(-100, -100),
			m: Point::new(-100, -100),
			drag: false,

			grid: Grid {
				show: true,
				size: Vector::new(16, 16),
				offset: Vector::new(-6, -6),
			},

			current: CurrentTool::Freehand,
			prim: Primitive::new(),
			bucket: Bucket::new(),
			freehand: Freehand::new(),
			dropper: EyeDropper::new(),
			editor: Editor::new(sprite),

			created: false,
		}
	}

	pub fn input(&mut self, ev: Input<i16>) {
		if self.editor.sprite().as_receiver().is_lock() {
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

	fn set_mouse(&mut self, p: Point<i16>) -> Point<i16> {
		self.mouse = Point::from_coordinates((p - self.pos) / self.zoom);
		self.mouse
	}

	pub fn zoom_from_center(&mut self, y: i16) {
		let p = self.editor.size();
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

	pub fn redo(&mut self) {
		self.editor.redo();
	}

	pub fn undo(&mut self) {
		self.editor.undo();
	}

	pub fn color(&self) -> u32 {
		let m = self.editor.sprite();
		let m = m.as_receiver();
		m.palette[m.color.get()]
	}

	pub fn pal(&self, color: u8) -> u32 {
		let m = self.editor.sprite();
		m.as_receiver().palette[color]
	}

	pub fn color_index(&self) -> u8 {
		let m = self.editor.sprite();
		m.as_receiver().color.get()
	}

	pub fn recreate(&mut self, m: ImageCell<'a>) {
		self.editor.image = m;
		self.editor.sync();
		self.created = false;
	}

	pub fn draw(&mut self, render: &mut ui::Render) {
		let red = 0xFF4136_FFu32;

		if !self.created {
			self.created = true;
			let m = self.editor.sprite();
			let m = m.as_receiver();
			let (w, h) = (m.width as u32, m.height as u32);
			render.create_texture(EDITOR_SPRITE_ID, w, h);
			render.create_texture(EDITOR_PREVIEW_ID, w, h);
		}

		render.by_image(&[EDITOR_SPRITE_ID, EDITOR_PREVIEW_ID], |canvas, layer| {
			match layer {
			EDITOR_SPRITE_ID => if self.editor.redraw {
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
			EDITOR_PREVIEW_ID => {
				canvas.set_draw_color(color!(TRANSPARENT));
				canvas.clear();

				match self.current {
				CurrentTool::Freehand => {
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
				_ => (),
				}
			}
			_ => (),
			}
		});

		render.draw_image_zoom(EDITOR_SPRITE_ID, self.pos, self.zoom);
		render.draw_image_zoom(EDITOR_PREVIEW_ID, self.pos, self.zoom);

		self.grid.draw(render, self.pos, self.editor.size(), self.zoom);
	}
}