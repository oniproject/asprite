#![feature(step_trait)]

extern crate sdl2;
extern crate num_traits;
extern crate nalgebra as na;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};

use sdl2::gfx::primitives::DrawRenderer;

mod tool;

mod common;
use common::*;

mod bb;
mod mask;

mod ui;
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


fn create_pal() -> Palette<Color> {
	const C_EMPTY: Color = Color { r: 0, g: 0, b: 0, a: 0 };
	const GB0: Color = Color { r: 202, g: 220, b: 159, a: 0xFF };
	const GB1: Color = Color { r:  15, g:  56, b:  15, a: 0xFF };
	const GB2: Color = Color { r:  48, g:  98, b:  48, a: 0xFF };
	const GB3: Color = Color { r: 139, g: 172, b:  15, a: 0xFF };
	const GB4: Color = Color { r: 155, g: 188, b:  15, a: 0xFF };

	let mut pal = Palette ([C_EMPTY; 256]);
	pal[0] = GB0;
	pal[1] = GB1;
	pal[2] = GB2;
	pal[3] = GB3;
	pal[4] = GB4;

	pal
}

const FONT_PATH: &str = "f/TerminusTTF-4.46.0.ttf";

macro_rules! rect(
	($x:expr, $y:expr, $w:expr, $h:expr) => (
		sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
	)
);

// macro_rules! point(
// 	($x:expr, $y:expr) => (
// 		sdl2::rect::Point::new($x as i32, $y as i32)
// 	)
// );

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

fn main() {
	let sdl_context = sdl2::init().unwrap();
	let video_subsys = sdl_context.video().unwrap();
	let ttf_context = sdl2::ttf::init().unwrap();

	let (w, h) = video_subsys.display_bounds(0)
		.unwrap().size();

	let mut window = video_subsys.window("rust-sdl2_gfx: draw line & FPSManager", w, h)
		.position_centered()
		.resizable()
		.maximized()
		.build()
		.unwrap();

	window.hide();
	window.show();

	let mut canvas = window.into_canvas()
		.software()
		.build().unwrap();
	let creator = canvas.texture_creator();

	canvas.set_draw_color(Color::RGB(100, 10, 100));
	canvas.clear();
	canvas.present();

	let cur = create_cursor();
	cur.set();
	let cur = Cursor::from_system(SystemCursor::Crosshair).unwrap();
	cur.set();

	// Load a font
	let font = ttf_context.load_font(FONT_PATH, 12).unwrap();
	//font.set_style(sdl2::ttf::STYLE_BOLD);

	let mut events = sdl_context.event_pump().unwrap();

	// let brush = mask::Mask::new_square(64, 64);

	let mut draw = bb::BB {
		grid: vec![false; (120 * 100) as usize],
		zoom: 6,
		pos: Point::new(200, 100),
		size: Point::new(120, 100),
		pts: Vec::new(),
		last: None,
		drawing: false,
	};

	let mut texture = creator
		.create_texture_target(PixelFormatEnum::RGBA8888, draw.size.x as u32, draw.size.y as u32)
		.unwrap();


	let mut paint_color = 0u8;

	let mut render = RenderSDL { canvas, font };

	let palette = create_pal();

	let gray0 = Color::RGB(0x19, 0x19, 0x19);
	let gray1 = Color::RGB(0x26, 0x26, 0x26);
	// let gray2 = Color::RGB(0x4F, 0x4F, 0x4F);

	//let white = Color::RGB(0xFF, 0xFF, 0xFF);
	let green = Color::RGB(0x00, 0xFF, 0x00);
	let red = Color::RGB(0xFF, 0x00, 0x00);

	// let black = Color::RGB(0x00, 0x00, 0x00);

	let frame = Frame {
		r: Rect::with_coords(10, 30, 100, 70),
		style: FrameStyle {
			normal: gray0,
			hovered: green,
			active: red,
		},
		state: State::Normal,
	};

	let mut paint = |draw: &bb::BB, paint_color| {
		render.canvas.set_draw_color(gray1);
		render.canvas.clear();

		render.canvas.with_texture_canvas(&mut texture, |canvas| {
			canvas.set_draw_color(palette[0]);
			canvas.clear();

			// TODO redraw only some area
			for (idx, is) in draw.grid.iter().enumerate() {
				if *is {
					let x = idx as i16 % draw.size.x;
					let y = idx as i16 / draw.size.x;
					canvas.pixel(x, y, palette[paint_color]).unwrap();
				}
			}

			for g in &draw.pts {
				let c = if g.active { green } else { red };
				canvas.pixel(g.x, g.y, c).unwrap();
			}
		}).unwrap();

		let size = draw.size * draw.zoom;

		let dst = Some(rect!(draw.pos.x, draw.pos.y, size.x, size.y));
		render.canvas.copy(&texture, None, dst).unwrap();

		{
			frame.draw(&render);
			render.text(frame.r, Align::Center, red, &"Hello Rust!");
		}

		render.canvas.present();
	};

	let mut drag = false;
	let mut update = true;
	'main: loop {
		if update {
			paint(&draw, paint_color);
		}
		update = false;

		for event in events.poll_iter() {
			match event {
				Event::MouseMotion {x, y, xrel, yrel, ..} => {
					if drag {
						draw.pos.x += xrel as i16;
						draw.pos.y += yrel as i16;
						update = true;
					} else {
						let x = (x as i16 - draw.pos.x) / draw.zoom;
						let y = (y as i16 - draw.pos.y) / draw.zoom;
						update = draw.update(Point::new(x, y));
					}
				}

				Event::Quit {..} => break 'main,

				Event::KeyDown {keycode: Some(keycode), ..} => {
					match keycode {
						Keycode::Escape => break 'main,
						Keycode::Num1 => paint_color = 1,
						Keycode::Num2 => paint_color = 2,
						Keycode::Num3 => paint_color = 3,
						Keycode::Num4 => paint_color = 4,
						_ => (),
					}
					update = true;
				}

				Event::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
					drag = true;
					update = true;
				}
				Event::MouseButtonUp { mouse_btn: MouseButton::Right, .. } => {
					drag = false;
					update = true;
				}

				Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
					draw.down();
					update = true;
				}
				Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => {
					draw.up();
					update = true;
				}

				Event::MouseWheel { y, ..} => {
					let last = draw.zoom;
					draw.zoom += y as i16;
					if draw.zoom < 1 { draw.zoom = 1 }
					if draw.zoom > 16 { draw.zoom = 16 }
					let diff = last - draw.zoom;

					draw.pos.x += draw.size.x * diff / 2;
					draw.pos.y += draw.size.y * diff / 2;

					update = true;
				}

				Event::Window { win_event, .. } => {
					println!("win_event {:?}", win_event);
					update = true;
				}

				_ => (),
			}
		}
	}
}
