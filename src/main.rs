#![feature(step_trait)]

extern crate image;
extern crate undo;
extern crate sdl2;
extern crate num_traits;
extern crate nalgebra as na;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};

use sdl2::render::Texture;

use sdl2::gfx::primitives::DrawRenderer;

mod common;
mod tool;
mod ui;
mod mask;
mod editor;
mod cmd_page;
mod sprite;
mod flood_fill;

//mod tilemap;

use common::*;
use tool::*;
use ui::*;

/*
 * FFDE12
 * FF2F7C
 * BEF600
 * 00ADBC
 * 9639AD
 *
 * 262626
 * 191919
 * 4F4F4F
 */


fn create_pal(pal: &mut Palette<u32>) {
	const GB0: u32 = 0xCADC9F_FF;
	const GB1: u32 = 0x0F380F_FF;
	const GB2: u32 = 0x306230_FF;
	const GB3: u32 = 0x8BAC0F_FF;
	const GB4: u32 = 0x9BBC0F_FF;

	pal[0] = GB0;
	pal[1] = GB1;
	pal[2] = GB2;
	pal[3] = GB3;
	pal[4] = GB4;
}

const FONT_PATH: &str = "f/TerminusTTF-4.46.0.ttf";

macro_rules! rect(
	($x:expr, $y:expr, $w:expr, $h:expr) => (
		sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
	)
);

use sdl2::mouse::{Cursor, SystemCursor};

fn create_cursor() -> Cursor {
	let data: [u8; 8] = [
		0b_00010000_u8,
		0b_00010000_u8,
		0b_00010000_u8,
		0b_11101110_u8,
		0b_00010000_u8,
		0b_00010000_u8,
		0b_00010000_u8,
		0b_00000000_u8,
	];

	let mask: [u8; 8] = [
		0b_00111000_u8,
		0b_00111000_u8,
		0b_11111110_u8,
		0b_11101110_u8,
		0b_11111110_u8,
		0b_00111000_u8,
		0b_00111000_u8,
		0b_00000000_u8,
	];

	Cursor::new(&data[..], &mask[..], 8, 8, 4, 4).unwrap()
}

struct Tilemap {
	data: Vec<usize>,
	width: usize,
	height: usize,
	tiles: Vec<mask::Mask>,
}

impl Tilemap {
	fn draw<F: Fn(usize, usize, bool)>(&self, f: F) {
		for y in 0..self.height {
			for x in 0..self.width {
				let idx = x + y * self.width;
				let tile = &self.tiles[self.data[idx]];
				tile.draw(|mx, my, is| {
					let x = mx as usize + x * 16;
					let y = my as usize + y * 16;
					f(x, y, is)
				})
			}
		}
	}

	fn draw_tilemap<F: Fn(usize, usize, bool)>(&self, f: F) {
		for (idx, tile) in self.tiles.iter().enumerate() {
			let x = idx % 8;
			let y = idx / 8;
			tile.draw(|mx, my, is| {
				let x = mx as usize + x * 16;
				let y = my as usize + y * 16;
				f(x, y, is)
			})
		}
	}

	fn at(&mut self, p: Point<i16>) -> Option<usize> {
		let x = p.x as usize;
		let y = p.y as usize;
		let idx = x + y * self.width;
		self.data.get(idx).map(|&t| t)
	}

	fn set(&mut self, p: Point<i16>, tile: usize) {
		let x = p.x as usize;
		let y = p.y as usize;
		if x >= self.width || y >= self.height {
			return;
		}
		let idx = x + y * self.width;
		self.data[idx] = tile;
	}
}

fn load_tilemap(width: usize, height: usize, fill: usize) -> Tilemap {
	use std::path::Path;
	use image::GenericImage;

	const ONE: &str = "tileset_1bit.png";
	const TWO: &str = "extra-1bits.png";

	let mut map = Tilemap {
		width, height,
		data: vec![fill; width*height],
		tiles: Vec::new(),
	};

	let mut one = image::open(&Path::new(ONE)).unwrap();
	let mut two = image::open(&Path::new(TWO)).unwrap();

	fn tiles(map: &mut Tilemap, m: &mut image::DynamicImage) {
		for y in 0..8 {
			for x in 0..8 {
				let sub = m.sub_image(x*16, y*16, 16, 16);
				let mut mask = mask::Mask::new_square(16, 16);
				for y in 0..16 {
					for x in 0..16 {
						let r = sub.get_pixel(x, y).data[0];
						let idx = x + y * 16;
						mask.pix[idx as usize] = r > 0;
					}
				}
				map.tiles.push(mask);
			}
		}
	}

	tiles(&mut map, &mut one);
	tiles(&mut map, &mut two);
	map
}


fn main() {
	let sdl_context = sdl2::init().unwrap();
	let video_subsys = sdl_context.video().unwrap();
	let ttf_context = sdl2::ttf::init().unwrap();

	let (w, h) = video_subsys.display_bounds(0)
		.unwrap().size();


	let mut window = video_subsys.window("ASprite", w, h)
		.position_centered()
		.resizable()
		.maximized()
		.build()
		.unwrap();

	window.hide();
	window.show();

	let mut ctx = window.into_canvas()
		.software()
		.build().unwrap();
	let creator = ctx.texture_creator();

	ctx.set_draw_color(Color::RGB(100, 10, 100));
	ctx.clear();
	ctx.present();

	let cur = create_cursor();
	cur.set();
	let cur = Cursor::from_system(SystemCursor::Crosshair).unwrap();
	cur.set();

	// Load a font
	let font = ttf_context.load_font(FONT_PATH, 12).unwrap();
	//font.set_style(sdl2::ttf::STYLE_BOLD);

	let mut events = sdl_context.event_pump().unwrap();

	// let brush = mask::Mask::new_square(64, 64);

	let mut sprite = sprite::Sprite::new(120, 100);
	create_pal(&mut sprite.palette);

	let mut texture = creator
		.create_texture_target(PixelFormatEnum::RGBA8888, sprite.width as u32, sprite.height as u32)
		.unwrap();

	let mut render = RenderSDL { ctx, font,
		last: 0, hot: 0, active: 0, kbd: 0,
		key: None,
		next_rect: Rect::with_coords(0, 0, 0, 0),
		mouse: (false, Point::new(0, 0)),
	};

	let (winx, winy) = render.ctx.output_size().unwrap();

	let mut app = App {
		update: true,
		quit: false,
		drag: false,
		drawing: false,

		tile: 0,

		statusbar: Rect::with_size(0, winy as i16 - 20, winx as i16, 20),
		freehand: freehand::Freehand {
			mode: freehand::Mode::PixelPerfect,
			last: Point::new(0, 0),
			pts: Vec::new(),
			color: 0,
		},
		editor: editor::Editor::new(6, Point::new(200, 100), sprite),
		map: load_tilemap(40, 30, 63),
	};

	while !app.quit {
		if app.update {
			app.paint(&mut texture, &mut render);
		}

		for event in events.poll_iter() {
			app.event(event, &mut render);
		}
	}
}

struct App<'a> {
	update: bool,
	quit: bool,
	drag: bool,
	drawing: bool,
	statusbar: Rect<i16>,
	freehand: freehand::Freehand,
	editor: editor::Editor<'a>,

	tile: usize,
	map: Tilemap,
}

impl<'a> App<'a> {
	fn paint<'t>(&mut self, texture: &mut Texture<'t>, render: &mut RenderSDL<sdl2::video::Window>) {
		//let gray0 = Color::RGB(0x19, 0x19, 0x19);
		let gray1 = Color::RGB(0x22, 0x22, 0x22);

		// let gray2 = Color::RGB(0x4F, 0x4F, 0x4F);

		// let green = 0x00FF00_FF;
		let red =   0xFF4136_FF;

		self.update = false;

		render.ctx.set_draw_color(gray1);
		render.ctx.clear();

		render.ctx.with_texture_canvas(texture, |canvas| {
			// TODO: redraw only changed area
			canvas.set_draw_color(Color::RGBA(0xCA, 0xDC, 0x9F, 0xFF));
			canvas.clear();

			let image = self.editor.image.as_receiver();
			for (frame, layers) in image.data.iter().enumerate() {
				for (layer, page) in layers.iter().enumerate() {
					let page = if layer == self.editor.layer && frame == self.editor.frame {
						&self.editor.canvas
					} else {
						page
					};
					for (idx, c) in page.page.iter().enumerate() {
						let x = idx % image.width;
						let y = idx / image.width;
						canvas.pixel(x as i16, y as i16, image.palette[*c].to_be()).unwrap();
					}
				}
			}

			// tool preview
			for g in &self.freehand.pts {
				let p = g.pt;
				if g.active {
					canvas.pixel(p.x, p.y, image.palette[self.freehand.color].to_be()).unwrap();
				} else {
					canvas.pixel(p.x, p.y, red).unwrap();
				}
			}

			// tool preview
			for g in &self.freehand.pts {
				let p = g.pt;
				if g.active {
					canvas.pixel(p.x, p.y, image.palette[self.freehand.color].to_be()).unwrap();
				} else {
					canvas.pixel(p.x, p.y, red).unwrap();
				}
			}

			// preview brush
			canvas.pixel(self.editor.mouse.x, self.editor.mouse.y, image.palette[self.editor.fg].to_be()).unwrap();
		}).unwrap();

		let size = self.editor.size() * self.editor.zoom;

		let dst = Some(rect!(self.editor.pos.x, self.editor.pos.y, size.x, size.y));
		render.ctx.copy(&texture, None, dst).unwrap();

		{ // ui
			render.prepare();
			let s = format!(" Freehand::{:?}  zoom:{}  [{:+} {:+}]  {:>3}#{:<3}",
				self.freehand.mode, self.editor.zoom, self.editor.mouse.x, self.editor.mouse.y, self.editor.fg, self.editor.bg);
			render.r(self.statusbar);
			render.label_bg(1, Align::Left, red, 0x001f3f_FF, &s);

			let r = Rect::with_size(10, 0, 100, 20);
			render.text(r, Align::Left, 0xFFFFFF_FF, "palette:");

			for i in 0..5 {
				render.r(Rect::with_size(10, 20 + 20*i as i16, 20, 20));
				render.btn_label(10 + i, &format!("{}", i), || {
					self.editor.fg = i as u8;
				});
			}

			let r = Rect::with_size(10, 120, 100, 20);
			render.text(r, Align::Left, 0xFFFFFF_FF, "freehand:");

			let modes = [
				freehand::Mode::Single,
				freehand::Mode::Discontinious,
				freehand::Mode::Continious,
				freehand::Mode::PixelPerfect,
				freehand::Mode::Line,

			];
			for (i, m) in modes.iter().enumerate() {
				render.r(Rect::with_size(10, 140 + 20*i as i16, 100, 20));
				render.btn_label(20 + i as u32, &format!("{:?}", m), || {
					self.freehand.mode = *m;
				});
			}

			{
				render.r(Rect::with_size(10, 260, 20, 20));
				render.btn_label(30, &"\u{2190}", || {
					self.editor.undo();
					self.update = true;
				});
				render.r(Rect::with_size(30, 260, 20, 20));
				render.btn_label(31, &"\u{2192}", || {
					self.editor.redo();
					self.update = true;
				});
			}

			render.finish();
		}

		let ww = 0xFFFFFF_FFu32.to_be();
		let bb = 0x000000_FFu32.to_be();

		self.map.draw(|x, y, is| {
			let c = if is { ww } else { bb };
			render.ctx.pixel(x as i16 + 600, y as i16, c).unwrap();
		});

		self.map.draw_tilemap(|x, y, is| {
			let c = if is { ww } else { bb };
			render.ctx.pixel(x as i16 + 600, y as i16 + 600, c).unwrap();
		});

		render.ctx.present();
	}

	fn event(&mut self, event: sdl2::event::Event, render: &mut RenderSDL<sdl2::video::Window>) {
		match event {
			Event::MouseMotion {x, y, xrel, yrel, ..} => {
				self.update = true;

				let p = Point::new(x as i16, y as i16);
				render.mouse.1 = p;

				let p = self.editor.set_mouse(p);
				if self.drag {
					self.editor.pos.x += xrel as i16;
					self.editor.pos.y += yrel as i16;
				} else if self.drawing {
					self.freehand.run(Input::Move(p), &mut self.editor);
				}
			}

			Event::Quit {..} => self.quit = true,

			Event::KeyDown { keycode: Some(keycode), keymod, ..} => {
				self.update = true;
				match keycode {
					Keycode::Escape => self.quit = true,
					Keycode::Num1 => self.editor.fg = 1,
					Keycode::Num2 => self.editor.fg = 2,
					Keycode::Num3 => self.editor.fg = 3,
					Keycode::Num4 => self.editor.fg = 4,

					Keycode::Num5 => self.freehand.mode = freehand::Mode::Single,
					Keycode::Num6 => self.freehand.mode = freehand::Mode::Discontinious,
					Keycode::Num7 => self.freehand.mode = freehand::Mode::Continious,
					Keycode::Num8 => self.freehand.mode = freehand::Mode::PixelPerfect,
					Keycode::U => self.editor.undo(),
					Keycode::R => self.editor.redo(),

					Keycode::Tab => {
						render.key = if keymod.contains(sdl2::keyboard::LSHIFTMOD) {
							Some(ui::Key::PrevWidget)
						} else {
							Some(ui::Key::NextWidget)
						};
					}
					_ => (),
				}
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
				self.drag = true;
				self.update = true;
			}
			Event::MouseButtonUp { mouse_btn: MouseButton::Right, .. } => {
				self.drag = false;
				self.update = true;
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
				let p = Point::new(x as i16, y as i16);
				render.mouse = (true, p);

				{
					let p = Point::from_coordinates((p - Point::new(600, 0)) / 16);
					self.map.set(p, self.tile);
				}
				{
					let p = Point::from_coordinates((p - Point::new(600, 600)) / 16);
					let r = Rect::with_size(0, 0, 8, 16);
					if r.contains(p) {
						println!("{}", p);
						self.tile = (p.x + p.y * 8) as usize;
					}
				}

				let p = self.editor.set_mouse(p);
				if p.x >= 0 && p.y >= 0 {
					self.freehand.run(Input::Press(p), &mut self.editor);
					self.drawing = true;
				}
				self.update = true;
			}
			Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
				let p = Point::new(x as i16, y as i16);
				render.mouse = (false, p);

				let p = self.editor.set_mouse(p);
				if p.x >= 0 && p.y >= 0 {
					self.freehand.run(Input::Release(p), &mut self.editor);
					self.drawing = false;
				}
				self.update = true;
			}

			Event::MouseWheel { y, ..} => {
				self.editor.zoom(y as i16);
				self.update = true;
			}

			Event::Window { win_event: sdl2::event::WindowEvent::Resized(w, h), .. } => {
				println!("resize {} {}", w, h);
				self.statusbar = Rect::with_size(0, h as i16 - 20, w as i16, 20);
				self.update = true;
			}

			_ => (),
		}
	}
}