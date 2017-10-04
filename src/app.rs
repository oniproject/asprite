use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use common::*;
use tool::*;
use ui;
use ui::*;
use editor::*;
use sprite::Sprite;

pub fn open_file() -> Option<String> {
	use nfd::{self, Response};

	let result = nfd::dialog().filter("gif").open().unwrap();

	let result = match result {
		Response::Okay(file) => Some(file),
		Response::OkayMultiple(files) => Some(files[0].clone()),
		Response::Cancel => None,
	};

	if let Some(filename) = result.clone() {
		// Open the file
		use std::fs::File;
		use gif;
		use gif::SetParameter;
		let file = File::open(filename).unwrap();

		let mut decoder = gif::Decoder::new(file);
		// Configure the decoder such that it will expand the image to RGBA.
		decoder.set(gif::ColorOutput::Indexed);
		// Read the file header
		let mut decoder = decoder.read_info().unwrap();
		println!("{}x{} has_pal: {} bg: {:?}",
			decoder.width(),
			decoder.height(),
			decoder.global_palette().is_some(),
			decoder.bg_color(),
		);
		while let Some(frame) = decoder.read_next_frame().unwrap() {
			// Process every frame
			println!("frame[{}]: {}x{} {}x{}, transparent: {:?} dis: {:?}",
				frame.palette.is_some(),
				frame.top, frame.left, frame.width, frame.height, frame.transparent, frame.dispose);
		}
	}

	result
}

pub struct App<'a> {
	pub update: bool,
	pub quit: bool,

	tools: Tools<'a>,
}

impl<'a> App<'a> {
	pub fn new(sprite: Sprite) -> Self {
		Self {
			update: true,
			quit: false,
			tools: Tools::new(6, Point::new(200, 100), sprite),
		}
	}

	pub fn paint(&mut self, render: &mut ui::Render) {
		self.update = false;

		let editor_bg = color!(WINDOW_BG);

		render.ctx.set_viewport(None);
		render.ctx.set_clip_rect(None);
		render.ctx.set_draw_color(editor_bg);
		render.ctx.clear();

		self.tools.draw(render);

		render.prepare();
		self.ui(render);
		self.update |= render.finish();

		render.ctx.present();
	}

	pub fn event(&mut self, event: sdl2::event::Event, render: &mut ui::Render) {
		self.update = true;
		match event {
			Event::MouseMotion {x, y, xrel, yrel, ..} => {
				let p = Point::new(x as i16, y as i16);
				let v = Vector::new(xrel as i16, yrel as i16);
				render.mouse.1 = p;
				self.tools.mouse_move(p, v);
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

					Keycode::Plus  | Keycode::KpPlus  => self.tools.zoom_from_center(1),
					Keycode::Minus | Keycode::KpMinus => self.tools.zoom_from_center(-1),

					Keycode::LShift |
					Keycode::RShift =>
						self.tools.input(Input::Special(true)),

					Keycode::LCtrl |
					Keycode::RCtrl =>
						self.tools.drag = true,

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

	pub fn ui<R: Immediate>(&mut self, ui: &mut R) {
		let r = ui.bounds();
		let width = r.dx();
		let height = r.dy();

		let menubar = Rect::with_size(0, 0, width, 20);
		let contextbar = Rect::with_size(0, 20, width, 40);
		let statusbar = Rect::with_size(0, height - 20, width, 20);
		let timeline = Rect::with_size(0, height - 20 - 200, width, 200);
		let palette = Rect::with_size(width-250, 60, 250, 500);

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

			let r = Rect::with_size(40, 0, 40, 40);
			for (i, &(icon, mode)) in modes.iter().enumerate() {
				ui.lay(r.x(40 * i as i16));
				let active = self.tools.current == mode;
				if ui.btn_icon(770 + i as u32, icon, active) {
					self.tools.current = mode;
				}
			}

			ui.lay(Rect::with_size(280, 12, 200, 14));
			match self.tools.current {
				CurrentTool::Freehand => {
					ui.checkbox_label(999, "perfect pixel", &mut self.tools.freehand.perfect);
				}
				CurrentTool::Primitive(_) => {
					ui.checkbox_label(999, "fill", &mut self.tools.prim.fill);
				}
				_ => (),
			}

			let zoom = Rect::with_size(ui.width() - 160, 10, 90, 20);
			ui.panel(zoom, |mut ui| {
				let label = Rect::new().wh(50, 20);
				let r = Rect::new().wh(20, 20);
				let r1 = r.x(51);
				let r2 = r.x(70);

				ui.lay(r1);
				ui.frame(BTN_BG, BTN_BORDER);
				ui.lay(r2);
				ui.frame(BTN_BG, BTN_BORDER);

				ui.lay(label);
				ui.label_left(&format!(" ZOOM {}", self.tools.zoom));
				ui.lay(r1);
				ui.btn_mini(30, "+", || self.tools.zoom_from_center(1));
				ui.lay(r2);
				ui.btn_mini(31, "-", || self.tools.zoom_from_center(-1));
			});

			let undoredo = Rect::with_size(ui.width() - 60, 10, 40, 20);
			ui.panel(undoredo, |mut ui| {
				let r1 = Rect::new().wh(20, 20);
				let r2 = r1.x(19);

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
			let tools = &self.tools;
			let r = ui.widget_rect();
			ui.text_center_left(r, STATUSBAR_COLOR, &format!(" zoom:{}  #{:<3}  [{:+} {:+}]",
				tools.zoom,
				tools.color_index(),
				tools.mouse.x, tools.mouse.y,
			));
		});

		ui.panel(timeline, |mut ui| {
			ui.clear(TIMELINE_BG);
			let mut sync = false;
			{
				let image = self.tools.editor.image.as_receiver();
				let count = image.data.len();
				for (i, layer) in image.data.iter().enumerate() {
					let y = 19 + 19 * (count - i - 1) as i16;
					let r = Rect::with_size(0, y, 20, 20);

					ui.lay(r.x(0));
					sync = sync || ui.checkbox_cell(70 + i as u32, &layer.visible);

					ui.lay(r.x(19));
					sync = sync || ui.checkbox_cell(90 + i as u32, &layer.lock);

					ui.lay(r.x(38).w(160));
					if ui.btn_label_left(100 + i as u32, &format!("  {}: {}", i, layer.name)) {
						println!("select layer: {}", i);
						image.layer.set(i);
						sync = true;
					}
				}
			}
			if sync {
				self.tools.editor.sync();
			}
		});

		ui.panel(palette, |mut ui| {
			ui.clear(BAR_BG);
			ui.header("Palette");

			let r = Rect::with_size(10, 40, 20, 20);
			for i in 0..5u8 {
				ui.lay(r.y(20*i as i16));
				let color = self.tools.pal(i);
				if ui.btn_color(1000 + i as u32, color) {
					self.tools.editor.change_color(i as u8);
				}
			}
		});

		ui.panel(menubar, |mut ui| {
			ui.clear(MENUBAR_BG);
			ui.label_left(" File  Edit  Select  View  Image  Layer  Tools  Help");
			let r = ui.widget(999);
			if ui.is_click() {
				use std::thread;
				thread::spawn(move || {
					println!("open file: {:?}", open_file());
				});
			}

			let w = r.dx();
			let h = r.dy();
			let tabs = Rect::with_size(200, 0, w - 200, h);
			ui.panel(tabs, |mut ui| {
				ui.lay(tabs.w(100));
				ui.frame(BAR_BG, None);
				ui.label_center("file.gif");
			})
		});
	}
}