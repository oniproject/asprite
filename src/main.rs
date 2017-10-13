#![feature(step_trait)]
#![feature(const_fn)]
#![feature(impl_trait)]
#![feature(conservative_impl_trait)]

extern crate ui;
extern crate image;
extern crate either;
extern crate rand;
extern crate undo;
extern crate sdl2;
extern crate num_traits;
extern crate nalgebra as na;
extern crate nfd;

use sdl2::mouse::Cursor;

use std::process;

#[macro_export] 
macro_rules! rect(
	($x:expr, $y:expr, $w:expr, $h:expr) => (
		$crate::sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
	)
);

#[macro_export] 
macro_rules! color(
	($c:expr) => (
		{
			use $crate::sdl2::gfx::primitives::ToColor;
			let (r, g, b, a) = $c.to_be().as_rgba();
			$crate::sdl2::pixels::Color::RGBA(r, g, b, a)
		}
	)
);

mod common;
mod tool;
mod gui;
mod mask;
mod editor;
mod cmd;
mod sprite;
mod grid;

mod app;

use common::*;
use gui::*;

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

const FONT_PATH: &str = "./res/TerminusTTF-4.46.0.ttf";


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

	// XXX: resize hack
	window.hide();
	window.show();

	let ctx = window.into_canvas()
		// XXX: glitch with .software()
		.software()
		.build().unwrap();

	let mut events = sdl_context.event_pump().unwrap();

	let sprite = {
		let mut sprite = sprite::Sprite::new("GEN", 160, 120);
		sprite.add_layer("Layer Down");
		sprite.add_layer("Layer 2");
		sprite.add_layer("Layer 3");
		sprite.add_layer("Layer 4");
		sprite.add_layer("Layer Up");

		create_pal(&mut sprite.palette);

		if true {
			let page = sprite.page_mut(0, 0);

			let r = Rect::with_size(0i32, 0, 160, 120);
			let va = Point::new(20i32, 10);
			let vb = Point::new(130i32, 100);

			gradient::draw_gradient(r, va, vb, |p, idx, total| {
				let pos = gradient::extra_dithered(idx, p.x as i16, p.y as i16, total, 5, 1);
				let ii = p.x + p.y * page.width as i32;
				page.page[ii as usize] = pos as u8;
			});
		}
		sprite
	};

	let font = ttf_context.load_font(FONT_PATH, FONT_HEIGHT).unwrap();

	let creator = ctx.texture_creator();
	let mut render = Render::new(ctx, &creator, font);

	render.graph.load_texture(ICON_TOOL_FREEHAND, "./res/tool_freehand.png");
	render.graph.load_texture(ICON_TOOL_FILL, "./res/tool_fill.png");
	render.graph.load_texture(ICON_TOOL_CIRC, "./res/tool_circ.png");
	render.graph.load_texture(ICON_TOOL_RECT, "./res/tool_rect.png");
	render.graph.load_texture(ICON_TOOL_PIP, "./res/tool_pip.png");

	let mut app = app::App::new(sprite);

	let mut main_loop = || {
		if app.quit {
			process::exit(1);
		}
		if app.update {
			app.paint(&mut render);
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