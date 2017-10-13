use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use common::*;
use tool::*;
use gui;
use gui::*;
use editor::*;
use cmd::*;

use undo::record::Record;
use std::rc::Rc;
use std::cell::RefCell;

pub struct App<'a> {
	pub update: bool,
	pub quit: bool,

	pub files: Vec<ImageCell<'a>>,
	pub current: usize,

	tools: Tools<'a>,
}

impl<'a> App<'a> {
	pub fn new(sprite: Sprite) -> Self {
		let files = vec![
			Rc::new(RefCell::new(Record::new(sprite))),
		];
		
		let mut tools = Tools::new(6, Point::new(300, 300), files[0].clone());
		tools.editor.sync();
		Self {
			update: true,
			quit: false,
			current: 0,
			files,
			tools,
		}
	}

	pub fn paint(&mut self, render: &mut gui::Render) {
		render.prepare(WINDOW_BG);
		self.tools.paint(render);
		self.ui(render);
		self.update = render.finish();
	}

	pub fn event(&mut self, event: sdl2::event::Event, render: &mut gui::Render) {
		self.update = true;
		match event {
		Event::MouseMotion {x, y, xrel, yrel, ..} => {
			let p = Point::new(x as i16, y as i16);
			render.mouse(Mouse::Move(p));
			let p = Point::new(x as i32, y as i32);
			let v = Vector::new(xrel as i32, yrel as i32);
			self.tools.mouse_move(p, v);
		}

		Event::Quit {..} => self.quit = true,

		Event::KeyUp { keycode: Some(keycode), .. } => {
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
			let _ctrl = keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD);
			match keycode {
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

				Keycode::Tab if shift => render.key = Some(gui::Key::PrevWidget),
				Keycode::Tab if !shift => render.key = Some(gui::Key::NextWidget),

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
			render.mouse(Mouse::Press(p));
			let p = Point::new(x as i32, y as i32);
			self.tools.mouse_press(p);
		}
		Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
			let p = Point::new(x as i16, y as i16);
			render.mouse(Mouse::Release(p));
			let p = Point::new(x as i32, y as i32);
			self.tools.mouse_release(p);
		}

		Event::MouseWheel { y, ..} => { self.tools.zoom_from_mouse(y as i32); }

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
		let palette = Rect::with_size(0, 60 + 50, 250, 500);

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
				if ui.btn_icon(770 + i as WidgetId, icon, active) {
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
				ui.label_left(&format!("ZOOM {}", self.tools.zoom));
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
			let r = ui.widget_rect().inset_x(INSET_X);
			ui.text_center_left(r, STATUSBAR_COLOR, &format!("zoom:{}  #{:<3}  [{:+} {:+}]",
				tools.zoom,
				tools.color_index(),
				tools.mouse.x, tools.mouse.y,
			));
		});

		ui.panel(timeline, |mut ui| {
			ui.clear(TIMELINE_BG);
			let mut sync = false;
			{
				let image = self.tools.editor.sprite();
				let image = image.as_receiver();
				let count = image.data.len();
				for (i, layer) in image.data.iter().enumerate() {
					let y = 19 + 19 * (count - i - 1) as i16;
					let r = Rect::with_size(8, y, 20, 20);

					ui.lay(r.x(0));
					sync = sync || ui.checkbox_cell(70 + i as WidgetId, &layer.visible);

					ui.lay(r.x(19));
					sync = sync || ui.checkbox_cell(90 + i as WidgetId, &layer.lock);

					ui.lay(r.x(38).w(160));
					if i == image.layer.get() {
						ui.frame(BTN_ACTIVE, None);
					}
					if ui.btn_label_left(100 + i as WidgetId, &format!("{}: {}", i, layer.name)) {
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

			let w = (palette.dx() - 5 * 2) / 20;
			let r = Rect::with_size(5, 40, 20, 20);
			for i in 0..256 {
				let x = i % w;
				let y = i / w;
				ui.lay(r.xy(20*x as i16, 20*y as i16));
				let color = self.tools.pal(i as u8);
				if ui.btn_color(1000 + i as WidgetId, color) {
					self.tools.editor.change_color(i as u8);
				}
			}
		});

		ui.panel(menubar, |mut ui| {
			ui.clear(MENUBAR_BG);
			ui.label_left("Open File");
			let w = ui.width();
			let h = ui.height();

			ui.lay(Rect::new().wh(40, h));
			let _r = ui.widget(998);
			if ui.is_click() {
				if let Some(sprite) = open_file().and_then(|f| load_sprite(&f)) {
					self.files.push(Rc::new(RefCell::new(Record::new(sprite))));
				}
			}

			let tabs = Rect::with_size(40, 0, w - 200, h);
			ui.panel(tabs, |mut ui| {
				let r = tabs.w(100); 
				for (i, sprite) in self.files.iter().enumerate() {
					let r = r.x(100 * i as i16);
					ui.lay(r);
					ui.widget(555 + i as WidgetId);
					let bg = if self.current == i { Some(BAR_BG) } else { None };
					if let Some(bg) = ui.switch(bg, BTN_BG, BTN_ACTIVE) {
						ui.frame(bg, None);
					}
					if ui.is_click() {
						self.current = i;
						self.tools.recreate(sprite.clone());
					}
					let m = sprite.borrow();
					let m = m.as_receiver();
					ui.label_left(&m.name);
				}
			})
		});
	}
}