use common::*;

#[derive(Clone, Debug)]
pub enum Command<N: Signed, C: Copy> {
	Line(Point<N>, Point<N>, C),
	Border(Rect<N>, C),
	Fill(Rect<N>, C),
	Clip(Option<Rect<N>>),
	Text(String, Point<N>, C),
	Image(usize, Point<N>),
}

pub trait Graphics<N: Signed, C: Copy> {
	fn command(&mut self, cmd: Command<N, C>);
	fn text_size(&mut self, s: &str) -> (u32, u32);
	fn image_size(&mut self, id: usize) -> (u32, u32);

	fn channel(&mut self, ch: usize);

	fn line(&mut self, a: Point<N>, b: Point<N>, color: C) {
		self.command(Command::Line(a, b, color));
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

	fn image(&mut self, m: usize, p: Point<N>) {
		self.command(Command::Image(m, p));
	}
	fn image_rect_center(&mut self, m: usize, r: Rect<N>) {
		self.image_align(m, r, 0.5, 0.5);
	}

	fn image_align(&mut self, m: usize, r: Rect<N>, x: f32, y: f32) {
		let (tw, th) = self.image_size(m);
		let size = Point::new(N::from(tw).unwrap(), N::from(th).unwrap());
		let p = align32(r, x, y, size);
		self.image(m, p);
	}

	fn text(&mut self, p: Point<N>, color: C, s: &str) {
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
		let size = Point::new(N::from(tw).unwrap(), N::from(th).unwrap());
		let p = align32(r, x, y, size);
		self.text(p, color, s);
	}

	fn render_frame<A, B>(&mut self, r: Rect<N>, bg: A, border: B)
		where
			A: Into<Option<C>>,
			B: Into<Option<C>>,
	{
		if let Some(bg) = bg.into() {
			self.fill(r, bg);
		}
		if let Some(border) = border.into() {
			self.border(r, border);
		}
	}
}