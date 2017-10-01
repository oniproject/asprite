use super::*;
use common::*;

pub struct Panel<'a, R: Immediate + 'a> {
	pub render: &'a mut R,
	pub r: Rect<i16>,
}

impl<'a, R: Immediate + 'a> Panel<'a, R> {
}

impl<'a, R: Immediate + 'a> Graphics<i16, u32> for Panel<'a, R> {
	fn command(&mut self, cmd: Command<i16, u32>) { self.render.command(cmd) }
	fn text_size(&mut self, s: &str) -> (u32, u32) { self.render.text_size(s) }
}

impl<'a, R: Immediate + 'a> Immediate for Panel<'a, R> {
	fn bounds(&self) -> Rect<i16> { self.r }

	fn widget(&mut self, id: u32) -> Rect<i16> { self.render.widget(id) }

	fn widget_rect(&self) -> Rect<i16> { self.render.widget_rect() }

	fn is_hot(&self) -> bool { self.render.is_hot() }
	fn is_active(&self) -> bool { self.render.is_active() }
	fn is_click(&self) -> bool { self.render.is_click() }
	fn lay(&mut self, r: Rect<i16>) {
		let r = self.r.min_translate_rect(r);
		self.render.lay(r);
	}
}

pub trait Immediate: Sized + Graphics<i16, u32> {
	fn widget(&mut self, id: u32) -> Rect<i16>;
	fn widget_rect(&self) -> Rect<i16>;

	fn bounds(&self) -> Rect<i16>;
	fn width(&self) -> i16 { self.bounds().w() }
	fn height(&self) -> i16 { self.bounds().h() }

	// fn bounds(&self) -> Rect<i16>;

	fn is_hot(&self) -> bool;
	fn is_active(&self) -> bool;
	fn is_click(&self) -> bool;

	fn lay(&mut self, r: Rect<i16>);

	fn run<F: FnOnce(Self)>(self, f: F) { f(self) }

	fn clear(&mut self, color: u32) {
		let r = self.bounds();
		self.lay(Rect::with_size(0, 0, r.w(), r.h()));
		self.fill(r, color);
	}

	fn panel(&mut self, r: Rect<i16>) -> Panel<Self> {
		Panel {
			render: self,
			r,
		}
	}

	fn frame<A, B>(&mut self, bg: A, border: B)
		where
			A: Into<Option<u32>>,
			B: Into<Option<u32>>,
	{
		let r = self.widget_rect();
		if let Some(bg) = bg.into() {
			self.fill(r, bg);
		}
		if let Some(border) = border.into() {
			self.border(r, border);
		}
	}

	fn label_right(&mut self, color: u32, text: &str) {
		let r = self.widget_rect();
		self.text_center_right(r, color, text)
	}
	fn label_center(&mut self, color: u32, text: &str) {
		let r = self.widget_rect();
		self.text_center(r, color, text)
	}
	fn label_left(&mut self, color: u32, text: &str) {
		let r = self.widget_rect();
		self.text_center_left(r, color, text)
	}

	fn btn_color(&mut self, id: u32, color: u32) -> bool {
		let r = self.widget(id);
		self.fill(r, color);
		self.is_click()
	}

	fn btn_mini<F: FnMut()>(&mut self, id: u32, label: &str, active: u32, mut cb: F) {
		let r = self.widget(id);

		if self.is_hot() && self.is_active() {
			self.fill(r, active);
		};

		let label_color = 0xFFFFFF_FF;

		// FIXME: fucking hack
		{
			let mut r = r.clone();
			let w = r.w() + 2;
			let h = r.h() + 2;
			r.set_w(w);
			r.set_h(h);
			self.text_center(r, label_color, label);
		}

		if self.is_click() {
			cb();
			println!("click: {}", label);
		}
	}

	fn btn_label<F: FnMut()>(&mut self, id: u32, label: &str, mut cb: F) {
		let r = self.widget(id);

		let bg = 0x353D4B_FF;
		let active_color = 0x0076FF_FF;

		if self.is_hot() {
			let bg = if self.is_active() { active_color } else { bg };
			self.fill(r, bg);
		};
		self.border(r, bg);

		let label_color = 0xECECEC_FF;

		// FIXME: fucking hack
		{
			let mut r = r.clone();
			let w = r.w() + 2;
			let h = r.h() + 2;
			r.set_w(w);
			r.set_h(h);
			self.text_center(r, label_color, label);
		}

		if self.is_click() {
			cb();
		}
	}
}