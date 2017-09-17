#![feature(step_trait)]

extern crate sdl2;
extern crate num_traits;
extern crate nalgebra as na;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};

use sdl2::gfx::primitives::DrawRenderer;

mod common;
mod tool;
mod ui;
mod mask;
mod editor;

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

	let mut draw = editor::SimpleContext::new(6, Point::new(200, 100), Point::new(120, 100));

	let mut freehand = freehand::Freehand {
		mode: freehand::Mode::PixelPerfect,
		last: Point::new(0, 0),
		pts: Vec::new(),
		color: 0,
	};

	let mut texture = creator
		.create_texture_target(PixelFormatEnum::RGBA8888, draw.size.x as u32, draw.size.y as u32)
		.unwrap();

	let mut render = RenderSDL { ctx, font };

	let palette = create_pal();

	let gray0 = Color::RGB(0x19, 0x19, 0x19);
	let gray1 = Color::RGB(0x26, 0x26, 0x26);
	// let gray2 = Color::RGB(0x4F, 0x4F, 0x4F);

	let green = Color::RGB(0x00, 0xFF, 0x00);
	let red = Color::RGB(0xFF, 0x00, 0x00);

	let (winx, winy) = render.ctx.output_size().unwrap();

	let statusbar = Frame {
		r: Rect::with_size(0, winy as i16 - 20, winx as i16, 20),
		style: FrameStyle {
			normal: gray0,
			hovered: green,
			active: red,
		},
		state: State::Normal,
	};

	let mut paint = |draw: &editor::SimpleContext, freehand: &freehand::Freehand| {
		render.ctx.set_draw_color(gray1);
		render.ctx.clear();

		render.ctx.with_texture_canvas(&mut texture, |canvas| {
			canvas.set_draw_color(palette[draw.bg]);
			canvas.clear();

			// TODO redraw only some area
			for (idx, c) in draw.grid.iter().enumerate() {
				let x = idx as i16 % draw.size.x;
				let y = idx as i16 / draw.size.x;
				canvas.pixel(x, y, palette[*c]).unwrap();
			}

			for g in &freehand.pts {
				let c = if g.active { green } else { red };
				canvas.pixel(g.pt.x, g.pt.y, c).unwrap();
			}

			// preview brush
			canvas.pixel(draw.mouse.x, draw.mouse.y, palette[draw.fg]).unwrap();
		}).unwrap();

		let size = draw.size * draw.zoom;

		let dst = Some(rect!(draw.pos.x, draw.pos.y, size.x, size.y));
		render.ctx.copy(&texture, None, dst).unwrap();

		{
			statusbar.draw(&render);
			let s = format!(" Freehand::{:?}  zoom:{}  [{:+} {:+}]  {:>3}#{:<3}",
				freehand.mode, draw.zoom, draw.mouse.x, draw.mouse.y, draw.fg, draw.bg);
			render.text(statusbar.r, Align::Left, red, &s);
		}

		render.ctx.present();
	};

	let mut drag = false;
	let mut drawing = false;
	let mut update = true;
	'main: loop {
		if update {
			paint(&draw, &freehand);
		}
		update = false;

		for event in events.poll_iter() {
			match event {
				Event::MouseMotion {x, y, xrel, yrel, ..} => {
					update = true;
					let p = draw.set_mouse(x, y);
					if drag {
						draw.pos.x += xrel as i16;
						draw.pos.y += yrel as i16;
					} else if drawing {
						freehand.run(Input::Move(p), &mut draw);
					}
				}

				Event::Quit {..} => break 'main,

				Event::KeyDown { keycode: Some(keycode), ..} => {
					match keycode {
						Keycode::Escape => break 'main,
						Keycode::Num1 => draw.fg = 1,
						Keycode::Num2 => draw.fg = 2,
						Keycode::Num3 => draw.fg = 3,
						Keycode::Num4 => draw.fg = 4,

						Keycode::Num5 => freehand.mode = freehand::Mode::Single,
						Keycode::Num6 => freehand.mode = freehand::Mode::Discontinious,
						Keycode::Num7 => freehand.mode = freehand::Mode::Continious,
						Keycode::Num8 => freehand.mode = freehand::Mode::PixelPerfect,
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

				Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
					let p = draw.set_mouse(x, y);
					freehand.run(Input::Press(p), &mut draw);
					drawing = true;
					update = true;
				}
				Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
					let p = draw.set_mouse(x, y);
					freehand.run(Input::Release(p), &mut draw);
					update = true;
					drawing = false;
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
