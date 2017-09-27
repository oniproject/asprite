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

use sdl2::render::{Texture, BlendMode};

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

use flood_fill::*;
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

const MAP_FILE: &str = "MAP.BIN";

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
				let tile = self.data[idx];
				self.tiles[tile].draw(|mx, my, is| {
					let x = mx as usize + x * 16;
					let y = my as usize + y * 16;
					f(x, y, is)
				})
			}
		}
	}

	fn draw_tilemap<F: Fn(usize, usize, bool, usize)>(&self, f: F) {
		for (idx, tile) in self.tiles.iter().enumerate() {
			let x = idx % 8;
			let y = idx / 8;
			tile.draw(|mx, my, is| {
				let x = mx as usize + x * 16;
				let y = my as usize + y * 16;
				f(x, y, is, idx)
			})
		}
	}

	fn get(&self, p: Point<i16>) -> Option<usize> {
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

impl flood_fill::Scanline<usize> for Tilemap {
	fn width(&self) -> usize { self.width }
	fn height(&self) -> usize { self.height }
	fn at(&self, x: i16, y: i16) -> usize {
		self.get(Point::new(x, y)).unwrap()
	}
	fn paint(&mut self, x: i16, y: i16, color: usize) {
		self.set(Point::new(x, y), color)
	}
}

impl Tilemap {
	fn load(width: usize, height: usize, fill: usize) -> Self {
		use std::path::Path;
		use image::GenericImage;

		use std::fs::File;
		use std::io::prelude::*;

		const ONE: &str = "tileset_1bit.png";
		const TWO: &str = "extra-1bits.png";

		let mut data = vec![fill; width*height];
		if let Ok(file) = File::open(MAP_FILE) {
			data.clear();
			for v in file.bytes().map(|v| v.unwrap()) {
				data.push(v as usize);
			}
		}

		let mut one = image::open(&Path::new(ONE)).unwrap();
		let mut two = image::open(&Path::new(TWO)).unwrap();

		let mut tiles = Vec::new();
		fn _tiles(tiles: &mut Vec<mask::Mask>, m: &mut image::DynamicImage) {
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
					tiles.push(mask);
				}
			}
		}

		_tiles(&mut tiles, &mut one);
		_tiles(&mut tiles, &mut two);

		Self {
			width, height, data, tiles,
		}
	}
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

	let mut texture_preview = creator
		.create_texture_target(PixelFormatEnum::RGBA8888, sprite.width as u32, sprite.height as u32)
		.unwrap();

	texture_preview.set_blend_mode(BlendMode::Blend);

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

		fill: false,

		tile: 0,

		statusbar: Rect::with_size(0, winy as i16 - 20, winx as i16, 20),
		freehand: freehand::Freehand {
			mode: freehand::Mode::PixelPerfect,
			last: Point::new(0, 0),
			pts: Vec::new(),
			color: 0,
			active: false,
			line: false,
		},
		editor: editor::Editor::new(6, Point::new(200, 100), sprite),
		map: Tilemap::load(40, 30, 63),
	};

	while !app.quit {
		if app.update {
			app.paint(&mut texture, &mut texture_preview, &mut render);
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
	fill: bool,
	map: Tilemap,
}

impl<'a> App<'a> {
	fn paint<'t>(&mut self, sprite: &mut Texture<'t>,  preview: &mut Texture<'t>, render: &mut RenderSDL<sdl2::video::Window>) {
		let editor_bg = Color::RGB(0x20, 0x24, 0x2F);
		let red =  0xFF4136_FFu32;

		self.update = false;

		render.ctx.set_draw_color(editor_bg);
		render.ctx.clear();


		enum Layer {
			Sprite,
			Preview,
		}
		let textures = [(sprite, Layer::Sprite), (preview, Layer::Preview)];

		render.ctx.with_multiple_texture_canvas(textures.iter(), |canvas, layer| {
			match *layer {
			Layer::Sprite => if self.editor.redraw {
				self.editor.redraw = false;

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
			}
			Layer::Preview => {
				canvas.set_draw_color(Color::RGBA(0x00, 0x00, 0x00, 0x00));
				canvas.clear();
				let image = self.editor.image.as_receiver();
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
			}
			}
		}).unwrap();

		let size = self.editor.size() * self.editor.zoom;

		let dst = Some(rect!(self.editor.pos.x, self.editor.pos.y, size.x, size.y));
		render.ctx.set_blend_mode(BlendMode::Blend);
		for t in &textures {
			render.ctx.copy(t.0, None, dst).unwrap();
		}
		render.ctx.set_blend_mode(BlendMode::None);

		{ // ui
			render.prepare();
			self.ui(render);
			if render.finish() {
				self.update = true;
			}
		}

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

			// grid
			let w = 16 * self.map.width as i16;
			for y in 0..self.map.height as i16 + 1 {
				render.ctx.hline(600, 600 + w, 16 * y, rr).unwrap();
			}
			let h = 16 * self.map.height as i16;
			for x in 0..self.map.width as i16 + 1 {
				render.ctx.vline(16 * x + 600, 0, h, rr).unwrap();
			}
		}

		render.ctx.present();
	}

	fn ui(&mut self, render: &mut RenderSDL<sdl2::video::Window>) {
		let statusbar_bg = 0x3F4350_FFu32;
		let statusbar_color = 0xA7A8AE_FFu32;
		let menubar_bg = 0x222833_FFu32;
		let color = 0xFFFFFF_FF;
		let bar_bg = 0x3f4957_FF;

		let cb_btn_border = 0x4E5763_FF;
		let cb_btn_bg = 0x3E4855_FF;
		let cb_btn_active = 0x0076FF_FF;

		let timeline_bg = 0x3a4351_FF;
		let header_bg = 0x525b68_FF;

		let width = self.statusbar.w();
		{
			// menubar
			let r = Rect::with_size(0, 0, width, 20);
			render.r(r);
			render.rect(r, menubar_bg);
			render.text(r, Align::Left, color, " File  Edit  Select  View  Image  Layer  Tools  Help");

			// contextbar 
			let r = Rect::with_size(0, 20, width, 40);
			render.rect(r, bar_bg);

			// undo/redo
			let r1 = Rect::with_size(40, 30, 20, 20);
			let r2 = Rect::with_size(60, 30, 20, 20);
			render.rect(r1, cb_btn_bg);
			render.rect(r2, cb_btn_bg);
			render.outline(r1, cb_btn_border);
			render.outline(r2, cb_btn_border);

			render.btn_mini(33, r1, &"\u{2190}", cb_btn_active, || self.editor.undo());
			render.btn_mini(34, r2, &"\u{2192}", cb_btn_active, || self.editor.redo());
		}

		let s = format!(" Freehand::{:?}  zoom:{}  [{:+} {:+}]  {:>3}#{:<3}",
			self.freehand.mode, self.editor.zoom, self.editor.mouse.x, self.editor.mouse.y, self.editor.fg, self.editor.bg);
		render.r(self.statusbar);
		render.label_bg(1, Align::Left, statusbar_color, statusbar_bg, &s);

		let rr = Rect::with_size(0, self.statusbar.max.y - self.statusbar.h() - 200, width, 200);
		render.rect(rr, timeline_bg);

		{
			let rr = Rect::with_size(width-250, 100, 250, 500);
			render.rect(rr, bar_bg);
			let rr = Rect::with_size(width-250, 100, 250, 20);
			render.rect(rr, header_bg);
			render.text(rr, Align::Right, color, &"\u{25BC} ");
		}

		{
			for i in 0..5u8 {
				render.r(Rect::with_size(width - 100, 20 + 20*i as i16, 20, 20));
				let image = self.editor.image.as_receiver();
				if render.btn_color(10 + i as u32, image.palette[i]) {
					self.editor.fg = i as u8;
				}
			}
		}

		let r = Rect::with_size(10, 120, 100, 20);
		render.text(r, Align::Left, 0xFFFFFF_FF, "freehand:");

		let modes = [
			freehand::Mode::Continious,
			freehand::Mode::PixelPerfect,
			freehand::Mode::Line,
			freehand::Mode::Rect,
		];
		for (i, m) in modes.iter().enumerate() {
			render.r(Rect::with_size(10, 140 + 20*i as i16, 100, 20));
			render.btn_label(20 + i as u32, &format!("{:?}", m), || {
				self.freehand.mode = *m;
			});
		}
	}

	fn event(&mut self, event: sdl2::event::Event, render: &mut RenderSDL<sdl2::video::Window>) {
		match event {
			Event::MouseMotion {x, y, xrel, yrel, ..} => {
				self.update = true;

				let p = Point::new(x as i16, y as i16);
				render.mouse.1 = p;

				if self.drawing {
					let p = Point::from_coordinates((p - Point::new(600, 0)) / 16);
					let r = Rect::with_size(0, 0, self.map.width as i16, self.map.height as i16);
					if r.contains(p) {
						self.map.set(p, self.tile);
					}
				}

				let p = self.editor.set_mouse(p);
				if self.drag {
					self.editor.pos.x += xrel as i16;
					self.editor.pos.y += yrel as i16;
				} else if self.drawing {
					self.freehand.run(Input::Move(p), &mut self.editor);
				}
			}

			Event::Quit {..} => self.quit = true,

			Event::KeyUp { keycode: Some(keycode), ..} => {
				self.update = true;
				match keycode {
					Keycode::LShift |
					Keycode::RShift => 
						self.freehand.run(Input::Special(false), &mut self.editor),
					_ => (),
				}
			}
			Event::KeyDown { keycode: Some(keycode), keymod, ..} => {
				self.update = true;
				let shift = keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD);
				let _alt = keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD);
				let ctrl = keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD);
				match keycode {
					Keycode::Escape => self.quit = true,

					Keycode::Space => self.fill = !self.fill,

					Keycode::Num1 => self.editor.fg = 1,
					Keycode::Num2 => self.editor.fg = 2,
					Keycode::Num3 => self.editor.fg = 3,
					Keycode::Num4 => self.editor.fg = 4,

					Keycode::Num7 => self.freehand.mode = freehand::Mode::Continious,
					Keycode::Num8 => self.freehand.mode = freehand::Mode::PixelPerfect,
					Keycode::Num9 => self.freehand.mode = freehand::Mode::Line,
					Keycode::LShift |
					Keycode::RShift => 
						self.freehand.run(Input::Special(true), &mut self.editor),

					Keycode::S if ctrl => {
						use std::fs::File;
						use std::io::prelude::*;
						println!("save map to {}", MAP_FILE);
						let mut file = File::create(MAP_FILE).expect("fail create file");
						for v in &self.map.data {
							let b = [*v as u8];
							file.write(&b).unwrap();
						}
					},

					Keycode::U => self.editor.undo(),
					Keycode::R => self.editor.redo(),

					Keycode::Tab if shift => render.key = Some(ui::Key::PrevWidget),
					Keycode::Tab if !shift => render.key = Some(ui::Key::NextWidget),

					_ => (),
				}
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Middle, .. } => {
				self.drag = true;
				self.update = true;
			}
			Event::MouseButtonUp { mouse_btn: MouseButton::Middle, .. } => {
				self.drag = false;
				self.update = true;
			}

			Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
				let p = Point::new(x as i16, y as i16);
				render.mouse = (true, p);

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