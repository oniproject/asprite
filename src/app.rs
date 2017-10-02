use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::mouse::{Cursor, SystemCursor};
use std::collections::HashMap;

use common::*;
use tool::*;
use ui;
use ui::*;
use editor::*;
use sprite::Sprite;

pub struct App<'a> {
	pub update: bool,
	pub quit: bool,

	tools: Tools<'a>,
	cursors: HashMap<SystemCursor, Cursor>,
}

impl<'a> App<'a> {
	pub fn new(sprite: Sprite) -> Self {
		let cursors = [
			SystemCursor::Arrow,
			SystemCursor::IBeam,
			SystemCursor::Wait,
			SystemCursor::Crosshair,
			SystemCursor::WaitArrow,
			SystemCursor::SizeNWSE,
			SystemCursor::SizeNESW,
			SystemCursor::SizeWE,
			SystemCursor::SizeNS,
			SystemCursor::SizeAll,
			SystemCursor::No,
			SystemCursor::Hand,
		];

		let cursors: HashMap<_, _> = cursors.iter().map(|&c| (c, Cursor::from_system(c).unwrap())).collect();
		cursors[&SystemCursor::Crosshair].set();

		Self {
			update: true,
			quit: false,

			cursors,
			tools: Tools::new(6, Point::new(200, 100), sprite),

			/*
			map: Tilemap::load(40, 30, 63),
			fill: false,
			tile: 0,
			*/
		}
	}
	pub fn paint(&mut self, render: &mut ui::Render) {
		self.update = false;

		let editor_bg = color!(WINDOW_BG);

		render.ctx.set_clip_rect(None);
		render.ctx.set_draw_color(editor_bg);
		render.ctx.clear();

		self.tools.draw(render);

		{ // ui
			render.prepare();
			self.ui(render);
			if render.finish() {
				self.update = true;
			}
			if render.hot != 0 {
				self.cursors[&SystemCursor::Hand].set();
			} else {
				self.cursors[&SystemCursor::Crosshair].set();
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

	pub fn ui<R: Immediate>(&mut self, ui: &mut R) {
		let r = ui.bounds();
		let width = r.w();
		let height = r.h();

		let menubar = Rect::with_size(0, 0, width, 20);
		let contextbar = Rect::with_size(0, 20, width, 40);
		let statusbar = Rect::with_size(0, height - 20, width, 20);
		let timeline = Rect::with_size(0, height - 20 - 200, width, 200);
		let palette = Rect::with_size(width-250, 60, 250, 500);

		ui.panel(menubar, |mut ui| {
			ui.clear(MENUBAR_BG);
			ui.label_left(LABEL_COLOR, " File  Edit  Select  View  Image  Layer  Tools  Help");
		});

		ui.panel(contextbar, |mut ui| {
			ui.clear(BAR_BG);

			ui.lay(Rect::with_size(10, 10, 20, 20));
			if ui.btn_color(32, self.tools.color()) {
				println!("xx");
			}

			let modes = [
				(ICON_TOOL_FREEHAND, CurrentTool::Freehand),
				(ICON_TOOL_FILL, CurrentTool::Bucket),
				(ICON_TOOL_CIRC, CurrentTool::Primitive(PrimitiveMode::Ellipse)),
				(ICON_TOOL_RECT, CurrentTool::Primitive(PrimitiveMode::Rect)),
				(ICON_TOOL_PIP, CurrentTool::EyeDropper),
			];

			for (i, &(icon, mode)) in modes.iter().enumerate() {
				ui.lay(Rect::with_size(40 + 32 * i as i16, 4, 32, 32));
				let active = self.tools.current == mode;
				if ui.btn_icon(770 + i as u32, icon, active) {
					self.tools.current = mode;
				}
			}

			match self.tools.current {
				CurrentTool::Freehand => {
					ui.lay(Rect::with_size(220, 12, 300, 14));
					ui.checkbox_label(999, " perfect pixel", &mut self.tools.freehand.perfect);
				}
				CurrentTool::Primitive(_) => {
					ui.lay(Rect::with_size(220, 12, 300, 14));
					ui.checkbox_label(999, " fill", &mut self.tools.prim.fill);
				}
				_ => (),
			}

			let zoom = Rect::with_size(ui.width() - 160, 10, 90, 20);
			ui.panel(zoom, |mut ui| {
				let label = Rect::with_size(0, 0, 50, 20);

				let r1 = Rect::with_size(51, 0, 20, 20);
				let r2 = Rect::with_size(70, 0, 20, 20);

				ui.lay(r1);
				ui.frame(BTN_BG, BTN_BORDER);
				ui.lay(r2);
				ui.frame(BTN_BG, BTN_BORDER);

				ui.lay(label);
				ui.label_left(LABEL_COLOR, &format!(" ZOOM {}", self.tools.zoom));
				ui.lay(r1);
				ui.btn_mini(30, "+", || self.tools.zoom_from_center(1));
				ui.lay(r2);
				ui.btn_mini(31, "-", || self.tools.zoom_from_center(-1));
			});

			let undoredo = Rect::with_size(ui.width() - 60, 10, 40, 20);
			ui.panel(undoredo, |mut ui| {
				// undo/redo
				let r1 = Rect::with_size(0, 0, 20, 20);
				let r2 = Rect::with_size(19, 0, 20, 20);

				ui.lay(r1);
				ui.frame(BTN_BG, BTN_BORDER);
				ui.lay(r2);
				ui.frame(BTN_BG, BTN_BORDER);

				ui.lay(r1);
				ui.btn_mini(33, "\u{2190}", || self.tools.undo());
				ui.lay(r2);
				ui.btn_mini(34, "\u{2192}", || self.tools.redo());
			});

		});

		ui.panel(statusbar, |mut ui| {
			ui.clear(STATUSBAR_BG);
			let freehand = &self.tools.freehand;
			let editor = &self.tools;
			ui.label_left(STATUSBAR_COLOR, &format!(" perfect pixel: {} zoom:{}  #{:<3}  [{:+} {:+}]",
				freehand.perfect, editor.zoom,
				editor.editor.image().color,
				editor.mouse.x, editor.mouse.y,
			));
		});

		ui.panel(timeline, |mut ui| {
			ui.clear(TIMELINE_BG);
			static mut SHOW: [bool; 5] = [true, false, true, false, false];
			static mut LOCK: [bool; 5] = [false, false, false, false, false];
			for j in 0..5 {
				let show = unsafe { &mut SHOW[j] };
				let lock = unsafe { &mut LOCK[j] };
				for (i, layer) in self.tools.editor.image().data.iter().enumerate() {
					let i = i + j;
					ui.lay(Rect::with_size(0, 19 + 19 * i as i16, 20, 20));
					ui.checkbox(70 + i as u32, show);
					ui.lay(Rect::with_size(19, 19 + 19 * i as i16, 20, 20));
					ui.checkbox(90 + i as u32, lock);

					ui.lay(Rect::with_size(38, 19 + 19 * i as i16, 160, 20));
					let r = ui.widget_rect();
					ui.border(r, BTN_BORDER);
					ui.label_left(LABEL_COLOR, &format!("  {}: {}", i, layer.name));
				}
			}
		});

		ui.panel(palette, |mut ui| {
			ui.clear(BAR_BG);
			ui.header(" Palette");

			for i in 0..5u8 {
				ui.lay(Rect::with_size(50, 40 + 20*i as i16, 20, 20));
				let color = self.tools.editor.image().palette[i];
				if ui.btn_color(100 + i as u32, color) {
					self.tools.editor.change_color(i as u8);
				}
			}
		});
	}

	pub fn event(&mut self, event: sdl2::event::Event, render: &mut ui::Render) {
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
				self.tools.mouse_move(p, xrel as i16, yrel as i16);
			}

			Event::Quit {..} => self.quit = true,

			Event::KeyUp { keycode: Some(keycode), ..} => {
				match keycode {
					Keycode::LShift |
					Keycode::RShift =>
						self.tools.input(Input::Special(false)),
					Keycode::LCtrl |
					Keycode::RCtrl =>
						self.tools.drag = false,
					_ => (),
				}
			}
			Event::KeyDown { keycode: Some(keycode), keymod, ..} => {
				let shift = keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD);
				let _alt = keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD);
				let ctrl = keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD);
				match keycode {
					Keycode::Q if ctrl => self.quit = true,
					Keycode::Escape => self.tools.input(Input::Cancel),

					// Keycode::Space => self.fill = !self.fill,

					// Keycode::Num1 => self.editor.fg = 1,
					// Keycode::Num2 => self.editor.fg = 2,
					// Keycode::Num3 => self.editor.fg = 3,
					// Keycode::Num4 => self.editor.fg = 4,

					// Keycode::Num7 => self.freehand.mode = freehand::Mode::Continious,
					// Keycode::Num8 => self.freehand.mode = freehand::Mode::PixelPerfect,
					// Keycode::Num9 => self.freehand.mode = freehand::Mode::Line,

					Keycode::Plus  | Keycode::KpPlus  => self.tools.zoom_from_center(1),
					Keycode::Minus | Keycode::KpMinus => self.tools.zoom_from_center(-1),

					Keycode::LShift |
					Keycode::RShift =>
						self.tools.input(Input::Special(true)),

					Keycode::LCtrl |
					Keycode::RCtrl =>
						self.tools.drag = true,

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

					Keycode::U => self.tools.undo(),
					Keycode::R => self.tools.redo(),

					Keycode::Tab if shift => render.key = Some(ui::Key::PrevWidget),
					Keycode::Tab if !shift => render.key = Some(ui::Key::NextWidget),

					_ => (),
				}
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Middle, .. } => {
				self.tools.drag = true;
			}
			Event::MouseButtonUp { mouse_btn: MouseButton::Middle, .. } => {
				self.tools.drag = false;
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
				let p = Point::new(x as i16, y as i16);
				render.mouse = (true, p);
				self.tools.mouse_press(p);

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
			}

			Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
				let p = Point::new(x as i16, y as i16);
				render.mouse = (false, p);
				self.tools.mouse_release(p);
			}

			Event::MouseWheel { y, ..} => { self.tools.zoom_from_mouse(y as i16); }

			// Event::Window { win_event: sdl2::event::WindowEvent::Resized(w, h), .. } => {
				//self.statusbar = Rect::with_size(0, h as i16 - 20, w as i16, 20);
			// }

			_ => (),
		}
	}
}