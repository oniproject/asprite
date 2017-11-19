use std::path::Path;
use super::*;

pub trait TextureManager {
	type RenderTarget;
	fn canvas<F: FnMut(&mut Self::RenderTarget, u32, u32)>(&mut self, id: usize, f: F);
	fn create_texture<T: Into<Option<usize>>>(&mut self, id: T, w: u32, h: u32) -> usize;
	fn load_texture<T: Into<Option<usize>>, P: AsRef<Path>>(&mut self, id: T, filename: P) -> usize;
}

pub trait TextSize {
	fn text_size(&mut self, s: &str) -> (u32, u32);
}
pub trait ImageSize {
	fn image_size(&mut self, id: usize) -> (u32, u32);
}

pub trait Graphics<N: BaseNum, C: Copy + 'static>: TextSize + ImageSize {
	fn fill(&mut self, r: Rect<N>, color: C);
	fn clip(&mut self, r: Option<Rect<N>>);
	fn text(&mut self, p: Point2<N>, color: C, s: &str);
	fn image_zoomed(&mut self, m: usize, p: Point2<N>, zoom: N);

	fn hline(&mut self, x1: N, x2: N, y: N, color: C) {
		self.fill(Rect::with_coords(x1, y, x2, y), color);
	}
	fn vline(&mut self, x: N, y1: N, y2: N, color: C) {
		self.fill(Rect::with_coords(x, y1, x, y2), color);
	}

	fn border(&mut self, r: Rect<N>, color: C) {
		let (x1, x2) = (r.min.x, r.max.x);
		self.hline(x1, x2, r.min.y, color);
		self.hline(x1, x2, r.max.y, color);
		let (y1, y2) = (r.min.y, r.max.y);
		self.vline(r.min.x, y1, y2, color);
		self.vline(r.max.x, y1, y2, color);
	}

	fn image(&mut self, m: usize, p: Point2<N>) {
		self.image_zoomed(m, p, N::one());
	}
	fn image_rect_center(&mut self, m: usize, r: Rect<N>) {
		self.image_align(m, r, 0.5, 0.5);
	}

	fn image_align(&mut self, m: usize, r: Rect<N>, x: f32, y: f32) {
		let (tw, th) = self.image_size(m);
		let size = Point2::new(N::from(tw).unwrap(), N::from(th).unwrap());
		let p = r.align(x, y, size);
		self.image(m, p);
	}

	fn text_center_left(&mut self, r: Rect<N>, color: C, s: &str) {
		self.text_align(r, 0.0, 0.5, color, s);
	}

	fn text_center_right(&mut self, r: Rect<N>, color: C, s: &str) {
		self.text_align(r, 1.0, 0.5, color, s);
	}

	fn text_center(&mut self, r: Rect<N>, color: C, s: &str) {
		self.text_align(r, 0.5, 0.5, color, s);
	}

	fn text_align(&mut self, r: Rect<N>, x: f32, y: f32, color: C, s: &str) {
		let (tw, th) = self.text_size(s);
		let size = Point2::new(N::from(tw).unwrap(), N::from(th).unwrap());
		let p = r.align(x, y, size);
		self.text(p, color, s);
	}
}

#[derive(Clone, Debug)]
pub enum Command<N: BaseNum, C: Copy + 'static> {
	Fill(Rect<N>, C),
	Clip(Option<Rect<N>>),
	Text(String, Point2<N>, C),
	Image(usize, Point2<N>, N),
}

pub trait GraphicsBase<N: BaseNum, C: Copy + 'static>: TextSize + ImageSize {
	fn command(&mut self, cmd: Command<N, C>);
	fn channel(&mut self, ch: usize);
}

impl<T, N: BaseNum, C: Copy + 'static> Graphics<N, C> for T
	where T: GraphicsBase<N, C>
{
	fn text(&mut self, p: Point2<N>, color: C, s: &str) {
		self.command(Command::Text(s.to_string(), p, color));
	}
	fn fill(&mut self, r: Rect<N>, color: C) {
		self.command(Command::Fill(r, color));
	}
	fn clip(&mut self, r: Option<Rect<N>>) {
		self.command(Command::Clip(r));
	}
	fn image_zoomed(&mut self, m: usize, p: Point2<N>, zoom: N) {
		self.command(Command::Image(m, p, zoom));
	}
}
