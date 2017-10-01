#![feature(step_trait)]
#![feature(const_fn)]

//extern crate image;
extern crate rand;
extern crate undo;
extern crate sdl2;
extern crate num_traits;
extern crate nalgebra as na;

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::BlendMode;
use sdl2::mouse::Cursor;

use std::process;

mod common;
mod tool;
mod ui;
mod mask;
mod editor;
mod cmd;
mod sprite;

mod app;
use app::*;

//mod tilemap;
use cmd::*;

use common::*;
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


fn _create_cursor() -> Cursor {
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

/*
const MAP_FILE: &str = "MAP.BIN";
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

impl Image<usize, u8> for Tilemap {
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
*/

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

	let ctx = window.into_canvas()
		// XXX: glitch with .software()
		.build().unwrap();


	// Load a font
	let font = ttf_context.load_font(FONT_PATH, FONT_HEIGHT).unwrap();
	//font.set_style(sdl2::ttf::STYLE_BOLD);

	let mut events = sdl_context.event_pump().unwrap();

	// let brush = mask::Mask::new_square(64, 64);

	let mut sprite = sprite::Sprite::new(160, 120);
	create_pal(&mut sprite.palette);

	if true {
		let page = sprite.page_mut(0, 0);

		let r = Rect::with_size(0i32, 0, 159, 119);
		let va = Point::new(20i32, 10);
		let vb = Point::new(130i32, 100);

		gradient::draw_gradient(r, va, vb, |p, idx, total| {
			let pos = gradient::extra_dithered(idx, p.x as i16, p.y as i16, total, 5, 1);
			let ii = p.x + p.y * page.width as i32;
			page.page[ii as usize] = pos as u8;
		});
	}

	let creator = ctx.texture_creator();

	let mut t1 = creator
		.create_texture_target(PixelFormatEnum::RGBA8888, sprite.width as u32, sprite.height as u32)
		.unwrap();
	let mut t2 = creator
		.create_texture_target(PixelFormatEnum::RGBA8888, sprite.width as u32, sprite.height as u32)
		.unwrap();

	t1.set_blend_mode(BlendMode::Blend);
	t2.set_blend_mode(BlendMode::Blend);


	let textures = [(&mut t1, Layer::Sprite), (&mut t2, Layer::Preview)];

	let mut render = Render::new(ctx, font);

	let _icon_tool_freehand = render.load_texture(&creator, "./res/tool_freehand.png");
	let _icon_tool_fill = render.load_texture(&creator, "./res/tool_fill.png");
	let _icon_tool_circ = render.load_texture(&creator, "./res/tool_circ.png");
	let _icon_tool_rect = render.load_texture(&creator, "./res/tool_rect.png");
	let _icon_tool_pip = render.load_texture(&creator, "./res/tool_pip.png");

	let mut app = App::new(sprite);

	let mut main_loop = || {
		if app.quit {
			process::exit(1);
		}
		if app.update {
			app.paint(&textures, &mut render);
		}
		if let Some(event) = events.wait_event_timeout(10) {
			app.event(event, &mut render);
		}
		for event in events.poll_iter() {
			app.event(event, &mut render);
		}
	};

	#[cfg(target_os = "emscripten")]
	emscripten::set_main_loop_callback(main_loop);
	#[cfg(not(target_os = "emscripten"))]
	loop { main_loop() }
}

#[cfg(target_os = "emscripten")]
mod emscripten {
	use std::cell::RefCell;
	use std::ptr::null_mut;
	use std::os::raw::{c_int, c_void, c_float};

	#[allow(non_camel_case_types)]
	type em_callback_func = unsafe extern fn();

	extern {
		pub fn emscripten_set_main_loop(func: em_callback_func, fps: c_int, simulate_infinite_loop: c_int);
		pub fn emscripten_cancel_main_loop();
		pub fn emscripten_get_now() -> c_float;
	}

	thread_local!(static MAIN_LOOP_CALLBACK: RefCell<*mut c_void> = RefCell::new(null_mut()));

	pub fn set_main_loop_callback<F>(callback: F) where F: FnMut() {
		MAIN_LOOP_CALLBACK.with(|log| {
			*log.borrow_mut() = &callback as *const _ as *mut c_void;
		});

		unsafe { emscripten_set_main_loop(wrapper::<F>, 0, 1); }

		unsafe extern "C" fn wrapper<F>() where F: FnMut() {
			MAIN_LOOP_CALLBACK.with(|z| {
				let closure = *z.borrow_mut() as *mut F;
				(*closure)();
			});
		}
	}
}