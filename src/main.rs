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

mod tilemap;

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

	let mut editor = editor::Editor::new(6, Point::new(200, 100), sprite);

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

	let mut paint = |render: &mut RenderSDL<sdl2::video::Window>, editor: &mut editor::Editor, freehand: &mut freehand::Freehand, statusbar: Rect<i16>| {
		let mut update = false;

		render.ctx.set_draw_color(gray1);
		render.ctx.clear();

		render.ctx.with_texture_canvas(&mut texture, |canvas| {
			// TODO: redraw only changed area
			canvas.set_draw_color(Color::RGBA(0xCA, 0xDC, 0x9F, 0xFF));
			canvas.clear();

			let image = editor.image.as_receiver();
			for (frame, layers) in image.data.iter().enumerate() {
				for (layer, page) in layers.iter().enumerate() {
					let page = if layer == editor.layer && frame == editor.frame {
						&editor.canvas
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
			for g in &freehand.pts {
				let p = g.pt;
				if g.active {
					canvas.pixel(p.x, p.y, image.palette[freehand.color].to_be()).unwrap();
				} else {
					canvas.pixel(p.x, p.y, red).unwrap();
				}
			}

			// tool preview
			for g in &freehand.pts {
				let p = g.pt;
				if g.active {
					canvas.pixel(p.x, p.y, image.palette[freehand.color].to_be()).unwrap();
				} else {
					canvas.pixel(p.x, p.y, red).unwrap();
				}
			}

			// preview brush
			canvas.pixel(editor.mouse.x, editor.mouse.y, image.palette[editor.fg].to_be()).unwrap();
		}).unwrap();

		let size = editor.size() * editor.zoom;

		let dst = Some(rect!(editor.pos.x, editor.pos.y, size.x, size.y));
		render.ctx.copy(&texture, None, dst).unwrap();

		{ // ui
			render.prepare();
			let s = format!(" Freehand::{:?}  zoom:{}  [{:+} {:+}]  {:>3}#{:<3}",
				freehand.mode, editor.zoom, editor.mouse.x, editor.mouse.y, editor.fg, editor.bg);
			render.label_bg(1, statusbar, Align::Left, red, 0x001f3f_FF, &s);

			let r = Rect::with_size(10, 0, 100, 20);
			render.text(r, Align::Left, 0xFFFFFF_FF, "palette:");

			for i in 0..5 {
				let r = Rect::with_size(10, 20 + 20*i as i16, 20, 20);
				render.btn_label(10 + i, r, &format!("{}", i), || {
					editor.fg = i as u8;
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
				let r = Rect::with_size(10, 140 + 20*i as i16, 100, 20);
				render.btn_label(20 + i as u32, r, &format!("{:?}", m), || {
					freehand.mode = *m;
				});
			}

			{
				let r = Rect::with_size(10, 240, 100, 20);
				render.text(r, Align::Left, 0xFFFFFF_FF, "undo/redo:");

				let r = Rect::with_size(10, 260, 20, 20);
				render.btn_label(30, r, &"\u{2190}", || {
					editor.undo();
					update = true;
				});
				let r = Rect::with_size(30, 260, 20, 20);
				render.btn_label(31, r, &"\u{2192}", || {
					editor.redo();
					update = true;
				});
			}

			render.finish();
		}

		render.ctx.present();
		update
	};

	let mut drag = false;
	let mut drawing = false;
	let mut update = true;

	let (winx, winy) = render.ctx.output_size().unwrap();
	let mut statusbar = Rect::with_size(0, winy as i16 - 20, winx as i16, 20);

	'main: loop {
		if true || update {
			update = paint(&mut render, &mut editor, &mut freehand, statusbar);
		}

		for event in events.poll_iter() {
			match event {
				Event::MouseMotion {x, y, xrel, yrel, ..} => {
					update = true;

					let p = Point::new(x as i16, y as i16);
					render.mouse.1 = p;

					let p = editor.set_mouse(p);
					if drag {
						editor.pos.x += xrel as i16;
						editor.pos.y += yrel as i16;
					} else if drawing {
						freehand.run(Input::Move(p), &mut editor);
					}
				}

				Event::Quit {..} => break 'main,

				Event::KeyDown { keycode: Some(keycode), ..} => {
					match keycode {
						Keycode::Escape => break 'main,
						Keycode::Num1 => editor.fg = 1,
						Keycode::Num2 => editor.fg = 2,
						Keycode::Num3 => editor.fg = 3,
						Keycode::Num4 => editor.fg = 4,

						Keycode::Num5 => freehand.mode = freehand::Mode::Single,
						Keycode::Num6 => freehand.mode = freehand::Mode::Discontinious,
						Keycode::Num7 => freehand.mode = freehand::Mode::Continious,
						Keycode::Num8 => freehand.mode = freehand::Mode::PixelPerfect,
						Keycode::U => editor.undo(),
						Keycode::R => editor.redo(),
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

					let p = editor.set_mouse(p);
					if p.x >= 0 && p.y >= 0 {
						freehand.run(Input::Press(p), &mut editor);
						drawing = true;
					}
					update = true;
				}
				Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
					let p = Point::new(x as i16, y as i16);
					render.mouse = (false, p);

					let p = editor.set_mouse(p);
					if p.x >= 0 && p.y >= 0 {
						freehand.run(Input::Release(p), &mut editor);
						drawing = false;
					}
					update = true;
				}

				Event::MouseWheel { y, ..} => {
					let last = editor.zoom;
					editor.zoom += y as i16;
					if editor.zoom < 1 { editor.zoom = 1 }
					if editor.zoom > 16 { editor.zoom = 16 }
					let diff = last - editor.zoom;

					editor.pos.x += editor.size().x * diff / 2;
					editor.pos.y += editor.size().y * diff / 2;

					update = true;
				}

				Event::Window { win_event: sdl2::event::WindowEvent::Resized(w, h), .. } => {
					println!("resize {} {}", w, h);
					statusbar = Rect::with_size(0, h as i16 - 20, w as i16, 20);
					update = true;
				}

				_ => (),
			}
		}
	}
}
