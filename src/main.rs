#![feature(step_trait)]

extern crate undo;
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
mod cmd_page;
mod sprite;

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

	let mut draw = editor::Editor::new(6, Point::new(200, 100), sprite);

	let mut freehand = freehand::Freehand {
		mode: freehand::Mode::PixelPerfect,
		last: Point::new(0, 0),
		pts: Vec::new(),
		color: 0,
	};

	let mut render = RenderSDL { ctx, font,
		last: 0, hot: 0, active: 0, kbd: 0,
		key: None,
		mouse: (false, Point::new(0, 0)),
	};

	//let gray0 = Color::RGB(0x19, 0x19, 0x19);
	let gray1 = Color::RGB(0x22, 0x22, 0x22);

	// let gray2 = Color::RGB(0x4F, 0x4F, 0x4F);

	// let green = 0x00FF00_FF;
	let red =   0xFF4136_FF;

	let (winx, winy) = render.ctx.output_size().unwrap();

	let statusbar = Rect::with_size(0, winy as i16 - 20, winx as i16, 20);

	let mut paint = |render: &mut RenderSDL<sdl2::video::Window>, draw: &mut editor::Editor, freehand: &mut freehand::Freehand| {
		render.ctx.set_draw_color(gray1);
		render.ctx.clear();

		render.ctx.with_texture_canvas(&mut texture, |canvas| {

			// TODO: redraw only changed area
			canvas.set_draw_color(Color::RGBA(0xCA, 0xDC, 0x9F, 0xFF));
			canvas.clear();

			let image = draw.image.as_receiver();
			for (frame, layers) in image.data.iter().enumerate() {
				for (layer, page) in layers.iter().enumerate() {
					let page = if layer == draw.layer && frame == draw.frame {
						&draw.canvas
					} else {
						page
					};
					for (idx, c) in page.page.iter().enumerate() {
						let x = idx % image.width;
						let y = idx / image.width;
						canvas.pixel(x as i16, y as i16, image.palette[*c]).unwrap();
					}
				}
			}

			// tool preview
			for g in &freehand.pts {
				let p = g.pt;
				if g.active {
					canvas.pixel(p.x, p.y, image.palette[freehand.color]).unwrap();
				} else {
					canvas.pixel(p.x, p.y, red).unwrap();
				}
			}

			// tool preview
			for g in &freehand.pts {
				let p = g.pt;
				if g.active {
					canvas.pixel(p.x, p.y, image.palette[freehand.color]).unwrap();
				} else {
					canvas.pixel(p.x, p.y, red).unwrap();
				}
			}

			// preview brush
			canvas.pixel(draw.mouse.x, draw.mouse.y, image.palette[draw.fg]).unwrap();
		}).unwrap();

		let size = draw.size() * draw.zoom;

		let dst = Some(rect!(draw.pos.x, draw.pos.y, size.x, size.y));
		render.ctx.copy(&texture, None, dst).unwrap();

		{
			// ui
			render.prepare();
			let s = format!(" Freehand::{:?}  zoom:{}  [{:+} {:+}]  {:>3}#{:<3}",
				freehand.mode, draw.zoom, draw.mouse.x, draw.mouse.y, draw.fg, draw.bg);
			render.label_bg(1, statusbar, Align::Left, red, 0x001f3f_FF, &s);

			let btns = 4;
			for i in 0..5 {
				let r = Rect::with_size(10, 20 + 20*i as i16, 21, 21);
				render.btn_label(btns+ i, r, &format!("{}", i), || {
					draw.fg = i as u8;
				});
			}

			let r = Rect::with_size(10, 120, 100, 20);
			render.text(r, Align::Left, 0xFFFFFF_FF, "freehand:");

			let modes = [
				freehand::Mode::Single,
				freehand::Mode::Discontinious,
				freehand::Mode::Continious,
				freehand::Mode::PixelPerfect,
			];
			for (i, m) in modes.iter().enumerate() {
				let r = Rect::with_size(10, 140 + 20*i as i16, 100, 21);
				render.btn_label(10 + i as u32, r, &format!("{:?}", m), || {
					freehand.mode = *m;
				});
			}

			render.finish();
		}

		render.ctx.present();
	};

	let mut drag = false;
	let mut drawing = false;
	let mut update = true;
	'main: loop {
		if true || update {
			paint(&mut render, &mut draw, &mut freehand);
		}
		update = false;

		for event in events.poll_iter() {
			match event {
				Event::MouseMotion {x, y, xrel, yrel, ..} => {
					update = true;

					let p = Point::new(x as i16, y as i16);
					render.mouse.1 = p;

					let p = draw.set_mouse(p);
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
						Keycode::U => draw.undo(),
						Keycode::R => draw.redo(),
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
					let p = Point::new(x as i16, y as i16);
					render.mouse = (true, p);

					let p = draw.set_mouse(p);
					freehand.run(Input::Press(p), &mut draw);
					drawing = true;
					update = true;
				}
				Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
					let p = Point::new(x as i16, y as i16);
					render.mouse = (false, p);

					let p = draw.set_mouse(p);
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

					draw.pos.x += draw.size().x * diff / 2;
					draw.pos.y += draw.size().y * diff / 2;

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
