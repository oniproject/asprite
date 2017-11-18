use common::*;
use tool::*;

use cmd::*;
use gui;
use gui::*;

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

	pub freehand: Freehand<i32, u8>,
	pub prim: Primitive<i32, u8>,
	pub bucket: Bucket<i32, u8>,
	pub dropper: EyeDropper<i32, u8>,

	pub pos: Point<i32>,
	pub grid: Grid,

	pub drag: bool,

	pub m: Point<i32>,
	pub mouse: Point<i32>,
	pub zoom: i32,

	pub created: bool,
}

impl<'a> Tools<'a> {
	pub fn new(zoom: i32, pos: Point<i32>, sprite: ImageCell<'a>) -> Self {
		Self {
			zoom, pos,
			mouse: Point::new(-100, -100),
			m: Point::new(-100, -100),
			drag: false,

			grid: Grid {
				show: true,
				rect: Rect::default(),
				size: Vector::new(16, 16),
				offset: Vector::new(-6, -6),
				zoom: zoom as i16,
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

	pub fn input(&mut self, ev: Input<i32>) {
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

	pub fn mouse_press(&mut self, p: Point<i32>) {
		let p = self.set_mouse(p);
		if p.x >= 0 && p.y >= 0 {
			self.input(Input::Press(p));
		}
	}

	pub fn mouse_release(&mut self, p: Point<i32>) {
		let p = self.set_mouse(p);
		if p.x >= 0 && p.y >= 0 {
			self.input(Input::Release(p));
		}
	}

	pub fn mouse_move(&mut self, p: Point<i32>, v: Vector<i32>) {
		let p = self.set_mouse(p);
		if self.drag {
			self.pos += v;
		} else {
			self.input(Input::Move(p));
		}
	}

	fn set_mouse(&mut self, p: Point<i32>) -> Point<i32> {
		self.mouse = Point::from_coordinates((p - self.pos) / self.zoom);
		self.mouse
	}

	pub fn zoom_from_center(&mut self, y: i32) {
		let p = self.editor.size();
		self.zoom(y, |diff| p * diff / 2);
	}

	pub fn zoom_from_mouse(&mut self, y: i32) {
		let p = self.mouse;
		self.zoom(y, |diff| p * diff);
	}

	fn zoom<F: FnOnce(i32) -> Point<i32>>(&mut self, y: i32, f: F) {
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

	pub fn paint(&mut self, render: &mut gui::Render) {
		let red = 0xFF4136_FFu32;

		if !self.created {
			self.created = true;
			let m = self.editor.sprite();
			let m = m.as_receiver();
			let (w, h) = (m.width as u32, m.height as u32);
			render.graph.create_texture(EDITOR_SPRITE_ID, w, h);
			render.graph.create_texture(EDITOR_PREVIEW_ID, w, h);
		}

		let redraw = self.editor.redraw;
		self.editor.redraw = None;

		if let Some(r) = redraw {
			render.graph.canvas(EDITOR_SPRITE_ID, |canvas, _, _| {
				let r = r.normalize();
				let clear_rect = rect!(r.min.x, r.min.y, r.dx(), r.dy());
				//canvas.set_clip_rect(r);

				let clear_color = color!(TRANSPARENT);
				// XXX let clear_color = color!(self.pal(0));
				canvas.set_draw_color(clear_color);
				canvas.draw_rect(clear_rect).unwrap();

				canvas.clear();

				self.editor.draw_pages(|page, palette| {
					let transparent = page.transparent;
					let br = Rect::with_size(0, 0, page.width as i32, page.height as i32);
					let r = br;
					blit(r, br, &page.page, |x, y, color| {
						let c = if Some(color) != transparent {
							palette[color].to_be()
						} else {
							TRANSPARENT
						};
						canvas.pixel(x as i16, y as i16, c).unwrap();
					})
				});
			});
		}

		render.graph.canvas(EDITOR_PREVIEW_ID, |canvas, _, _| {
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
					canvas.pixel(p.x as i16, p.y as i16, c).unwrap();
				}

				// preview brush
				canvas.pixel(
					self.mouse.x as i16, self.mouse.y as i16,
					self.color().to_be()).unwrap();
			}
			_ => (),
			}
		});

		let pos = Point::new(self.pos.x as i16, self.pos.y as i16);
		let zoom = self.zoom as i16;

		render.image_zoomed(EDITOR_SPRITE_ID, pos, zoom);
		render.image_zoomed(EDITOR_PREVIEW_ID, pos, zoom);

		self.grid.zoom = zoom;
		self.grid.rect = Rect {
			min: self.pos,
			max: Point::from_coordinates(self.pos.coords + self.editor.size().coords),
		};

		self.grid.paint(render);
	}
}