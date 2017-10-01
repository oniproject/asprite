use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::render::{Texture, BlendMode};
use sdl2::gfx::primitives::{DrawRenderer, ToColor};

use common::*;
use tool::*;
use ui;
use ui::*;
use editor::Editor;
use sprite::Sprite;

macro_rules! rect(
	($x:expr, $y:expr, $w:expr, $h:expr) => (
		sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
	)
);

#[derive(Clone, Copy, Debug)]
enum CurrentTool {
	Freehand,
	PixelPerfect,
	Bucket,
	EyeDropper,
	Primitive(PrimitiveMode),
}

struct Tools<'a> {
	current: CurrentTool,
	editor: Editor<'a>,

	freehand: Freehand<i16, u8>,
	prim: Primitive<i16, u8>,
	bucket: Bucket<i16, u8>,
	dropper: EyeDropper<i16, u8>,
}

impl<'a> Tools<'a> {
	pub fn input(&mut self, ev: Input<i16>) {
		match self.current {
			CurrentTool::Freehand => {
				self.freehand.perfect = false;
				self.freehand.run(ev, &mut self.editor);
			}
			CurrentTool::PixelPerfect => {
				self.freehand.perfect = true;
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

pub struct App<'a> {
	pub update: bool,
	pub quit: bool,
	drag: bool,

	tools: Tools<'a>,
}

impl<'a> App<'a> {
	pub fn new(sprite: Sprite) -> Self {
		Self {
			update: true,
			quit: false,
			drag: false,

			tools: Tools {
				current: CurrentTool::PixelPerfect,
				prim: Primitive::new(),
				bucket: Bucket::new(),
				freehand: Freehand::new(),
				dropper: EyeDropper::new(),
				editor: Editor::new(6, Point::new(200, 100), sprite),
			},

			/*
			map: Tilemap::load(40, 30, 63),
			fill: false,
			tile: 0,
			*/


		}
	}
	pub fn paint<'t>(&mut self, sprite: &mut Texture<'t>,  preview: &mut Texture<'t>, render: &mut ui::RenderSDL<sdl2::video::Window>) {
		self.update = false;

		let red =  0xFF4136_FFu32;

		let editor_bg = {
			let (r, g, b, a) = WINDOW_BG.to_be().as_rgba();
			Color::RGBA(r, g, b, a)
		};

		render.ctx.set_draw_color(editor_bg);
		render.ctx.clear();

		enum Layer {
			Sprite,
			Preview,
		}
		let textures = [(sprite, Layer::Sprite), (preview, Layer::Preview)];

		render.ctx.with_multiple_texture_canvas(textures.iter(), |canvas, layer| {
			match *layer {
			Layer::Sprite => if self.tools.editor.redraw {
				self.tools.editor.redraw = false;
				self.tools.editor.draw_pages(|page, stride, palette| {
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
				let image = self.tools.editor.image();

				// tool preview
				let freehand = &self.tools.freehand; 
				for &(p, active) in &freehand.pts {
					let c = if active {
						image.palette[freehand.color].to_be()
					} else {
						red
					};
					canvas.pixel(p.x, p.y, c).unwrap();
				}

				// preview brush
				let editor = &self.tools.editor;
				canvas.pixel(editor.mouse.x, editor.mouse.y, editor.fg().to_be()).unwrap();
			}
			}
		}).unwrap();

		{ // display image and preview

			let size = self.tools.editor.size();
			let src = rect!(0, 0, size.x, size.y);
			let zoom = self.tools.editor.zoom;
			let size = size * zoom;
			let pos = self.tools.editor.pos;
			let dst = rect!(pos.x, pos.y, size.x, size.y);

			render.ctx.set_blend_mode(BlendMode::Blend);
			for t in &textures {
				render.ctx.copy(t.0, src, dst).unwrap();
			}
			render.ctx.set_blend_mode(BlendMode::None);
		}

		{ // grid
			let rr = 0xFF0000_FFu32.to_be();
			let gg = 0x00FF00_FFu32.to_be();
			let (ox, oy) = (self.tools.editor.pos.x, self.tools.editor.pos.y);
			let size = self.tools.editor.size();
			let zoom = self.tools.editor.zoom;

			let grid_w = 16;
			let grid_h = 16;

			let (x1, x2) = (ox, ox + size.x * zoom);
			let (y1, y2) = (oy, oy + size.y * zoom);

			let ex = size.x / grid_w;
			let ey = size.y / grid_h;
			let ix = (size.x % grid_w != 0) as i16;
			let iy = (size.y % grid_h != 0) as i16;

			if true {
				for y in 1..ey + iy {
					let y = oy - 1 + (y * grid_h) * zoom;
					render.ctx.hline(x1, x2, y, rr).unwrap();
				}

				for x in 1..ex + ix {
					let x = ox - 1 + (x * grid_w) * zoom;
					render.ctx.vline(x, y1, y2, rr).unwrap();
				}
			}

			// canvas border
			render.ctx.hline(x1-1, x2, y1-1, gg).unwrap();
			render.ctx.hline(x1-1, x2, y2+0, gg).unwrap();
			render.ctx.vline(x1-1, y1-1, y2, gg).unwrap();
			render.ctx.vline(x2+0, y1-1, y2, gg).unwrap();
		}

		{ // ui
			render.prepare();
			self.ui(render);
			if render.finish() {
				self.update = true;
			}
		}

		/*
		if false { // tilemap
			let ww = 0xFFFFFF_FFu32.to_be();
			let bb = 0x000000_FFu32.to_be();
			let rr = 0xFF0000_FFu32.to_be();

			self.map.draw(|x, y, is| {
				let c = if is { ww } else { bb };
				render.ctx.pixel(x as i16 + 600, y as i16, c).unwrap();
			});
			self.map.draw_tilemap(|x, y, is, tile| {
				let c = if is { ww } else if tile == self.tile { rr } else { bb };
				render.ctx.pixel(x as i16 + 600, y as i16 + 600, c).unwrap();
			});

		}
		*/

		render.ctx.present();
	}

	pub fn ui<R: Immediate>(&mut self, render: &mut R) {
		let r = render.bounds();
		let width = r.w();
		let height = r.h();

		let menubar = Rect::with_size(0, 0, width, 20);
		let contextbar = Rect::with_size(0, 20, width, 40);
		let statusbar = Rect::with_size(0, height - 20, width, 20);
		let timeline = Rect::with_size(0, height - 20 - 200, width, 200);
		let palette = Rect::with_size(width-250, 60, 250, 500);
		let tools = Rect::with_size(10, 140, 200, 300);

		render.panel(menubar).run(|mut render| {
			render.clear(MENUBAR_BG);
			render.label_left(LABEL_COLOR, " File  Edit  Select  View  Image  Layer  Tools  Help");
		});

		render.panel(contextbar).run(|mut render| {
			render.clear(BAR_BG);

			// undo/redo
			let r1 = Rect::with_size(40, 10, 20, 20);
			let r2 = Rect::with_size(60, 10, 20, 20);

			render.lay(r1);
			render.frame(BTN_BG, BTN_BORDER);
			render.lay(r2);
			render.frame(BTN_BG, BTN_BORDER);

			render.lay(r1);
			render.btn_mini(33, &"\u{2190}", BTN_ACTIVE, || self.tools.editor.undo());
			render.lay(r2);
			render.btn_mini(34, &"\u{2192}", BTN_ACTIVE, || self.tools.editor.redo());
		});

		render.panel(statusbar).run(|mut render| {
			render.clear(STATUSBAR_BG);
			let freehand = &self.tools.freehand;
			render.label_left(STATUSBAR_COLOR, &format!(" perfect pixel: {} zoom:{}  {:>3}#{:<3}  [{:+} {:+}]",
				freehand.perfect, self.tools.editor.zoom,
				self.tools.editor.image().fg, self.tools.editor.image().bg,
				self.tools.editor.mouse.x, self.tools.editor.mouse.y,
			));
		});

		render.panel(timeline).run(|mut render| {
			render.clear(TIMELINE_BG);
			let w = render.width();
			for (i, layer) in self.tools.editor.image().data.iter().enumerate() {
				render.lay(Rect::with_size(0, 20 * i as i16, w, 20));
				render.label_left(LABEL_COLOR, &format!("  {}: {}", i, layer.name));
			}
		});

		render.panel(palette).run(|mut render| {
			render.clear(BAR_BG);

			let w = render.width();
			render.lay(Rect::with_size(0, 0, w, 20));
			render.frame(HEADER_BG, None);
			render.label_right(LABEL_COLOR, &"\u{25BC} ");
			render.label_left(LABEL_COLOR, &" Palette");

			for i in 0..5u8 {
				render.lay(Rect::with_size(50, 40 + 20*i as i16, 20, 20));
				let color = self.tools.editor.image().palette[i];
				if render.btn_color(10 + i as u32, color) {
					self.tools.editor.change_foreground(i as u8);
				}
			}
		});

		render.panel(tools).run(|mut render| {
			render.clear(BAR_BG);

			let w = render.width();
			render.lay(Rect::with_size(0, 0, w, 20));
			render.frame(HEADER_BG, None);
			render.label_right(LABEL_COLOR, &"\u{25BC} ");
			render.label_left(LABEL_COLOR, &" Tools");

			let modes = [
				CurrentTool::Freehand,
				CurrentTool::PixelPerfect,
				CurrentTool::Bucket,
				CurrentTool::EyeDropper,
				CurrentTool::Primitive(PrimitiveMode::DrawEllipse),
				CurrentTool::Primitive(PrimitiveMode::FillEllipse),
				CurrentTool::Primitive(PrimitiveMode::DrawRect),
				CurrentTool::Primitive(PrimitiveMode::FillRect),
			];
			for (i, m) in modes.iter().enumerate() {
				render.lay(Rect::with_size(10, 40 + 20 * i as i16, w - 20, 20));
				render.btn_label(21 + i as u32, &format!("{:?}", m), || {
					self.tools.current = *m;
				});
			}
		});
	}

	pub fn event(&mut self, event: sdl2::event::Event, render: &mut ui::RenderSDL<sdl2::video::Window>) {
		self.update = true;
		match event {
			Event::MouseMotion {x, y, xrel, yrel, ..} => {
				let p = Point::new(x as i16, y as i16);
				render.mouse.1 = p;
				/* 
				if self.drawing {
					let p = Point::from_coordinates((p - Point::new(600, 0)) / 16);
					let r = Rect::with_size(0, 0, self.map.width as i16, self.map.height as i16);
					if r.contains(p) {
						self.map.set(p, self.tile);
					}
				} */

				let p = self.tools.editor.set_mouse(p);
				if self.drag {
					self.tools.editor.pos.x += xrel as i16;
					self.tools.editor.pos.y += yrel as i16;
				} else {
					self.tools.input(Input::Move(p));
				}
			}

			Event::Quit {..} => self.quit = true,

			Event::KeyUp { keycode: Some(keycode), ..} => {
				match keycode {
					Keycode::LShift |
					Keycode::RShift => 
						self.tools.input(Input::Special(false)),
					_ => (),
				}
			}
			Event::KeyDown { keycode: Some(keycode), keymod, ..} => {
				let shift = keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD);
				let _alt = keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD);
				let _ctrl = keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD);
				match keycode {
					Keycode::Q => self.quit = true,
					Keycode::Escape => self.tools.input(Input::Cancel),

					// Keycode::Space => self.fill = !self.fill,

					// Keycode::Num1 => self.editor.fg = 1,
					// Keycode::Num2 => self.editor.fg = 2,
					// Keycode::Num3 => self.editor.fg = 3,
					// Keycode::Num4 => self.editor.fg = 4,

					// Keycode::Num7 => self.freehand.mode = freehand::Mode::Continious,
					// Keycode::Num8 => self.freehand.mode = freehand::Mode::PixelPerfect,
					// Keycode::Num9 => self.freehand.mode = freehand::Mode::Line,
					Keycode::LShift |
					Keycode::RShift => 
						self.tools.input(Input::Special(true)),

					/*
					Keycode::S if ctrl => {
						use std::fs::File;
						use std::io::prelude::*;
						println!("save map to {}", MAP_FILE);
						let mut file = File::create(MAP_FILE).expect("fail create file");
						for v in &self.map.data {
							let b = [*v as u8];
							file.write(&b).unwrap();
						}
					}
					*/

					Keycode::U => self.tools.editor.undo(),
					Keycode::R => self.tools.editor.redo(),

					Keycode::Tab if shift => render.key = Some(ui::Key::PrevWidget),
					Keycode::Tab if !shift => render.key = Some(ui::Key::NextWidget),

					_ => (),
				}
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Middle, .. } => {
				self.drag = true;
			}
			Event::MouseButtonUp { mouse_btn: MouseButton::Middle, .. } => {
				self.drag = false;
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
				let p = Point::new(x as i16, y as i16);
				render.mouse = (true, p);

				/*
				{
					let p = Point::from_coordinates((p - Point::new(600, 0)) / 16);
					let r = Rect::with_size(0, 0, self.map.width as i16, self.map.height as i16);
					if r.contains(p) {
						if self.fill {
							self.map.fill(p, self.tile);
						} else {
							self.map.set(p, self.tile);
						}
					}
				}
				{
					let p = Point::from_coordinates((p - Point::new(600, 600)) / 16);
					let r = Rect::with_size(0, 0, 8, 16);
					if r.contains(p) {
						self.tile = (p.x + p.y * 8) as usize;
					}
				}
				*/

				let p = self.tools.editor.set_mouse(p);
				if p.x >= 0 && p.y >= 0 {
					self.tools.input(Input::Press(p));
				}
			}

			Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
				let p = Point::new(x as i16, y as i16);
				render.mouse = (false, p);

				let p = self.tools.editor.set_mouse(p);
				if p.x >= 0 && p.y >= 0 {
					self.tools.input(Input::Release(p));
				}
			}

			Event::MouseWheel { y, ..} => { self.tools.editor.zoom(y as i16); }

			// Event::Window { win_event: sdl2::event::WindowEvent::Resized(w, h), .. } => {
				//self.statusbar = Rect::with_size(0, h as i16 - 20, w as i16, 20);
			// }

			_ => (),
		}
	}
}