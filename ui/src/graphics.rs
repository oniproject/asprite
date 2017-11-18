use std::path::Path;
use super::*;

#[derive(Clone, Debug)]
pub enum Command<N: BaseNum, C: Copy + 'static> {
	Line(Point2<N>, Point2<N>, C),
	Border(Rect<N>, C),
	Fill(Rect<N>, C),
	Clip(Option<Rect<N>>),
	Text(String, Point2<N>, C),
	Image(usize, Point2<N>, N),
}

pub trait TextureManager {
	type RenderTarget;
	fn canvas<F: FnMut(&mut Self::RenderTarget, u32, u32)>(&mut self, id: usize, f: F);
	fn create_texture<T: Into<Option<usize>>>(&mut self, id: T, w: u32, h: u32) -> usize;
	fn load_texture<T: Into<Option<usize>>, P: AsRef<Path>>(&mut self, id: T, filename: P) -> usize;
}

pub trait Graphics<N: BaseNum, C: Copy + 'static> {
	fn command(&mut self, cmd: Command<N, C>);
	fn text_size(&mut self, s: &str) -> (u32, u32);
	fn image_size(&mut self, id: usize) -> (u32, u32);
	fn channel(&mut self, ch: usize);

	fn line(&mut self, a: Point2<N>, b: Point2<N>, color: C) {
		self.command(Command::Line(a, b, color));
	}

	fn hline(&mut self, x1: N, x2: N, y: N, color: C) {
		self.line(Point2::new(x1, y), Point2::new(x2, y), color);
	}
	fn vline(&mut self, x: N, y1: N, y2: N, color: C) {
		self.line(Point2::new(x, y1), Point2::new(x, y2), color);
	}

	fn border(&mut self, r: Rect<N>, color: C) {
		self.command(Command::Border(r, color));
	}
	fn fill(&mut self, r: Rect<N>, color: C) {
		self.command(Command::Fill(r, color));
	}
	fn clip(&mut self, r: Option<Rect<N>>) {
		self.command(Command::Clip(r));
	}

	fn image(&mut self, m: usize, p: Point2<N>) {
		self.image_zoomed(m, p, N::one());
	}
	fn image_zoomed(&mut self, m: usize, p: Point2<N>, zoom: N) {
		self.command(Command::Image(m, p, zoom));
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

	fn text(&mut self, p: Point2<N>, color: C, s: &str) {
		self.command(Command::Text(s.to_string(), p, color));
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