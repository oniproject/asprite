#![feature(step_trait)]

extern crate sdl2;
extern crate num_traits;
extern crate nalgebra as na;

use std::env;
use std::path::Path;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::TextureQuery;
use sdl2::pixels::{self, Color, PixelFormatEnum};

use sdl2::gfx::primitives::DrawRenderer;

mod bipbuffer;
mod bre;
mod bb;

mod widget;
mod math;
use widget::*;
use math::*;

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

const GB0: Color = Color { r: 202, g: 220, b: 159, a: 0xFF };
const GB1: Color = Color { r:  15, g:  56, b:  15, a: 0xFF };
const GB2: Color = Color { r:  48, g:  98, b:  48, a: 0xFF };
const GB3: Color = Color { r: 139, g: 172, b:  15, a: 0xFF };
const GB4: Color = Color { r: 155, g: 188, b:  15, a: 0xFF };

const FONT_PATH: &str = "f/TerminusTTF-4.46.0.ttf";

macro_rules! rect(
	($x:expr, $y:expr, $w:expr, $h:expr) => (
		sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
	)
);

macro_rules! point(
	($x:expr, $y:expr) => (
		sdl2::rect::Point::new($x as i32, $y as i32)
	)
);

struct Mask {
	pix: Vec<bool>,
	w: usize,
	h: usize,
}

impl Mask {
	fn new_square(w: usize, h: usize) -> Self {
		Self {
			w, h,
			pix: vec![true; w*h],
		}
	}

	fn draw<F>(&self, at: Point<i16>, pixel: F)
		where F: Fn(i16, i16)
	{
		let ex = at.x + self.w as i16;
		let ey = at.y + self.h as i16;
		let mut ptr = self.pix.as_ptr();
		for y in at.y..ey {
			for x in at.x..ex {
				unsafe {
					if *ptr {
						pixel(x, y);
					}
					ptr = ptr.offset(1);
				}
			}
		}
	}
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

	// Load a font
	let mut font = ttf_context.load_font(FONT_PATH, 12).unwrap();
	//font.set_style(sdl2::ttf::STYLE_BOLD);

	let mut events = sdl_context.event_pump().unwrap();

	let brush = Mask::new_square(64, 64);

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

	let mut paint_color = GB1;

	use widget::Render as RR;
	let mut render = Render { canvas, font };

	let gray0 = Color::RGB(0x19, 0x19, 0x19);
	let gray1 = Color::RGB(0x26, 0x26, 0x26);
	let gray2 = Color::RGB(0x4F, 0x4F, 0x4F);

	//let white = Color::RGB(0xFF, 0xFF, 0xFF);
	let green = Color::RGB(0x00, 0xFF, 0x00);
	let red = Color::RGB(0xFF, 0x00, 0x00);

	let black = Color::RGB(0x00, 0x00, 0x00);

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
			canvas.set_draw_color(GB0);
			canvas.clear();

			// TODO redraw only some area
			for (idx, is) in draw.grid.iter().enumerate() {
				if *is {
					let x = idx as i16 % draw.size.x;
					let y = idx as i16 / draw.size.x;
					canvas.pixel(x, y, paint_color).unwrap();
				}
			}

			for g in &draw.pts {
				let c = if g.active { green } else { red };
				canvas.pixel(g.x, g.y, c).unwrap();
			}
		}).unwrap();

		let size = draw.size * draw.zoom;

		let dst = Some(rect!(draw.pos.x, draw.pos.y, size.x, size.y));
		render.canvas
			.copy(&texture, None, dst)
			.unwrap();

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
						Keycode::Num1 => paint_color = GB1,
						Keycode::Num2 => paint_color = GB2,
						Keycode::Num3 => paint_color = GB3,
						Keycode::Num4 => paint_color = GB4,
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

struct Render<'ttf_module, 'rwops, T: sdl2::render::RenderTarget> {
	canvas: sdl2::render::Canvas<T>,
	font: sdl2::ttf::Font<'ttf_module, 'rwops>,
}

//impl<'a, T: sdl2::render::RenderTarget + 'a> widget::Render for Render<'a, T> {
impl<'ttf_module, 'rwops> widget::Render for Render<'ttf_module, 'rwops, sdl2::video::Window> {
	type Color = Color;
	//fn bounds(&self) -> Rect<i16> { }

	fn pixel(&self, p: Point<i16>, color: Self::Color) {}
	fn line(&self, start: Point<i16>, end: Point<i16>, color: Self::Color) {}

	fn rect(&self, r: Rect<i16>, color: &Self::Color) {
		self.canvas.box_(
			r.min.x, r.min.y,
			r.max.x, r.max.y,
			*color).unwrap()
	}

	fn outline(&self, r: Rect<i16>, color: Self::Color) {}
	fn icon(&self, r: Rect<i16>, index: usize) {}
	fn text(&mut self, r: Rect<i16>, align: Align, color: Self::Color, s: &str) {
		// render a surface, and convert it to a texture bound to the canvas
		let surface = self.font
			.render(s)
			.blended(color)
			.unwrap();

		let creator = self.canvas.texture_creator();

		let texture = creator
			.create_texture_from_surface(&surface)
			.unwrap();

		let TextureQuery { width, height, .. } = texture.query();
		let rw = r.max.x - r.min.x;
		let rh = r.max.y - r.min.y;

		let addx = match align {
			Align::Left => 0,
			Align::Center => (rw - width as i16) / 2,
			Align::Right => rw - width as i16,
		};
		let addy = (rh - height as i16) / 2;

		let r = rect!(r.min.x + addx, r.min.y + addy, width, height);

		self.canvas
			.copy(&texture, None, Some(r))
			.unwrap();
	}

	//fn bezier(&self, vx: &[i16], vy: &[i16], s: i32, color: u32);
}
