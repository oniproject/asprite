use common::*;
use sdl2::gfx::primitives::DrawRenderer;

use sdl2;

pub enum Align {
	Left,
	Right,
	Center,
}

pub enum Event {
	Press,
	Release,

	MouseEnter,
	MouseLeave,
}

pub trait Render {
	type Color;
	//fn bounds(&self) -> Rect<i16>;

	fn pixel(&self, p: Point<i16>, color: &Self::Color);
	fn line(&self, start: Point<i16>, end: Point<i16>, color: &Self::Color);

	fn rect(&self, r: Rect<i16>, color: &Self::Color);
	fn outline(&self, r: Rect<i16>, color: &Self::Color);

	fn icon(&self, r: Rect<i16>, index: usize);
	fn text(&mut self, r: Rect<i16>, align: Align, color: Self::Color, s: &str);

	//fn bezier(&self, vx: &[i16], vy: &[i16], s: i32, color: u32);
}

pub struct RenderSDL<'ttf_module, 'rwops, T: sdl2::render::RenderTarget> {
	pub ctx: sdl2::render::Canvas<T>,
	pub font: sdl2::ttf::Font<'ttf_module, 'rwops>,
}

//impl<'a, T: sdl2::render::RenderTarget + 'a> widget::Render for Render<'a, T> {
impl<'ttf_module, 'rwops> Render for RenderSDL<'ttf_module, 'rwops, sdl2::video::Window> {
	type Color = sdl2::pixels::Color;
	//fn bounds(&self) -> Rect<i16> { }

	fn pixel(&self, p: Point<i16>, color: &Self::Color) {
		self.ctx.pixel(p.x, p.y, *color).unwrap()
	}
	fn line(&self, start: Point<i16>, end: Point<i16>, color: &Self::Color) {
		self.ctx.line(
			start.x, start.y,
			end.x, end.y,
			*color).unwrap()
	}

	fn rect(&self, r: Rect<i16>, color: &Self::Color) {
		self.ctx.box_(
			r.min.x, r.min.y,
			r.max.x, r.max.y,
			*color).unwrap()
	}

	fn outline(&self, r: Rect<i16>, color: &Self::Color) {
		self.ctx.rectangle(
			r.min.x, r.min.y,
			r.max.x, r.max.y,
			*color).unwrap()

	}

	fn icon(&self, _: Rect<i16>, _: usize) {
		unimplemented!()
	}

	fn text(&mut self, r: Rect<i16>, align: Align, color: Self::Color, s: &str) {
		// render a surface, and convert it to a texture bound to the canvas
		let surface = self.font
			.render(s)
			.blended(color)
			.unwrap();

		let creator = self.ctx.texture_creator();

		let texture = creator
			.create_texture_from_surface(&surface)
			.unwrap();

		let sdl2::render::TextureQuery { width, height, .. } = texture.query();
		let rw = r.max.x - r.min.x;
		let rh = r.max.y - r.min.y;

		let addx = match align {
			Align::Left => 0,
			Align::Center => (rw - width as i16) / 2,
			Align::Right => rw - width as i16,
		};
		let addy = (rh - height as i16) / 2;

		let r = sdl2::rect::Rect::new(
			(r.min.x + addx) as i32,
			(r.min.y + addy) as i32, width, height);

		self.ctx
			.copy(&texture, None, Some(r))
			.unwrap();
	}

	//fn bezier(&self, vx: &[i16], vy: &[i16], s: i32, color: u32);
}
