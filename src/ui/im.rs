use common::*;
use super::*;

pub struct Panel<'a, R: Immediate + 'a> {
	render: &'a mut R,
	r: Rect<i16>,
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

	fn is_hot(&self) -> bool;
	fn is_active(&self) -> bool;
	fn is_click(&self) -> bool;

	fn lay(&mut self, r: Rect<i16>);

	fn clear(&mut self, color: u32) {
		let r = self.bounds();
		self.lay(Rect::with_size(0, 0, r.w(), r.h()));
		self.fill(r, color);
	}
	fn header(&mut self, title: &str) {
		let w = self.width();
		self.lay(Rect::with_size(0, 0, w, 20));
		self.frame(HEADER_BG, None);
		self.label_right(LABEL_COLOR, "\u{25BC} ");
		self.label_left(LABEL_COLOR, title);
	}

	fn panel<F: FnOnce(Panel<Self>)>(&mut self, r: Rect<i16>, f: F) {
		let mut panel = Panel {
			render: self,
			r,
		};
		panel.lay(Rect::with_size(0, 0, r.w(), r.h()));
		f(panel);
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

	fn btn_icon(&mut self, id: u32, icon: usize, active: bool) -> bool {
		let r = self.widget(id);
		if active {
			self.fill(r, BTN_ACTIVE);
		}
		self.image(icon, r);
		self.is_click()
	}

	fn btn_mini<F: FnMut()>(&mut self, id: u32, label: &str, mut cb: F) {
		let r = self.widget(id);
		if self.is_hot() && self.is_active() {
			self.fill(r, BTN_ACTIVE);
		};
		self.text_center(r, LABEL_COLOR, label);
		if self.is_click() {
			cb();
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
		self.text_center(r, LABEL_COLOR, label);
		if self.is_click() {
			cb();
		}
	}


	fn checkbox(&mut self, id: u32, value: &mut bool) {
		let r = self.widget(id);
		if self.is_click() {
			*value = !*value;
		}
		self.draw_checkbox(r, *value);
	}

	fn checkbox_label(&mut self, id: u32, label: &str, value: &mut bool) {
		let r = self.widget(id);
		let check = r.clone().set_w(r.h());
		let lab = r.clone().pos(r.h(), 0);
		if self.is_click() {
			*value = !*value;
		}
		self.draw_checkbox(check, *value);
		self.text_center_left(lab, LABEL_COLOR, label)
	}

	fn draw_checkbox(&mut self, r: Rect<i16>, value: bool) {
		if value {
			self.fill(r, BTN_ACTIVE);
			self.text_center(r, LABEL_COLOR, "\u{2714}");
		} else {
			self.fill(r, BTN_BG);
			self.border(r, BTN_BORDER);
		}
	}
}