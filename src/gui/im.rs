use common::*;
use super::*;
use std::cell::Cell;

pub type WidgetId = u16;

pub struct Panel<'a, R: Immediate + 'a> {
	render: &'a mut R,
	r: Rect<i16>,
}

impl<'a, R: Immediate + 'a> Graphics<i16, u32> for Panel<'a, R> {
	fn command(&mut self, cmd: Command<i16, u32>) { self.render.command(cmd) }
	fn text_size(&mut self, s: &str) -> (u32, u32) { self.render.text_size(s) }
	fn image_size(&mut self, id: usize) -> (u32, u32) { self.render.image_size(id) }
	fn channel(&mut self, ch: usize) { self.render.channel(ch) }
}

impl<'a, R: Immediate + 'a> Immediate for Panel<'a, R> {
	fn bounds(&self) -> Rect<i16> { self.r }
	fn widget(&mut self, id: WidgetId) -> Rect<i16> { self.render.widget(id) }
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
	fn widget(&mut self, id: WidgetId) -> Rect<i16>;
	fn widget_rect(&self) -> Rect<i16>;

	fn bounds(&self) -> Rect<i16>;
	fn width(&self) -> i16 { self.bounds().dx() }
	fn height(&self) -> i16 { self.bounds().dy() }

	fn is_hot(&self) -> bool;
	fn is_active(&self) -> bool;
	fn is_click(&self) -> bool;

	fn lay(&mut self, r: Rect<i16>);

	fn clear(&mut self, color: u32) {
		let r = self.bounds();
		self.lay(Rect::with_size(0, 0, r.dx(), r.dy()));
		self.fill(r, color);
	}

	fn render_frame<A, B>(&mut self, r: Rect<i16>, bg: A, border: B)
		where
			A: Into<Option<u32>>,
			B: Into<Option<u32>>,
	{
		if let Some(bg) = bg.into() {
			self.fill(r, bg);
		}
		if let Some(border) = border.into() {
			self.border(r, border);
		}
	}
	fn header(&mut self, title: &str) {
		let w = self.width();
		let r = self.bounds().h(20);
		self.render_frame(r, HEADER_BG, None);

		let r = Rect::new().wh(w, 20);
		self.lay(r);
		self.label_right("\u{25BC}");
		self.label_left(title);
	}

	fn panel<F: FnOnce(Panel<Self>)>(&mut self, r: Rect<i16>, f: F) {
		let mut panel = Panel {
			render: self,
			r,
		};
		panel.lay(Rect::new().wh(r.dx(), r.dy()));
		f(panel);
	}

	fn frame<A, B>(&mut self, bg: A, border: B)
		where
			A: Into<Option<u32>>,
			B: Into<Option<u32>>,
	{
		let r = self.widget_rect();
		self.render_frame(r, bg, border);
	}

	fn label_right(&mut self, text: &str) {
		let r = self.widget_rect().inset_x(INSET_X);
		self.text_center_right(r, LABEL_COLOR, text)
	}
	fn label_center(&mut self, text: &str) {
		let r = self.widget_rect().inset_x(INSET_X);
		self.text_center(r, LABEL_COLOR, text)
	}
	fn label_left(&mut self, text: &str) {
		let r = self.widget_rect().inset_x(INSET_X);
		self.text_center_left(r, LABEL_COLOR, text)
	}

	fn btn_color(&mut self, id: WidgetId, color: u32) -> bool {
		let r = self.widget(id);
		self.fill(r, color);
		self.is_click()
	}

	fn btn_icon(&mut self, id: WidgetId, icon: usize, active: bool) -> bool {
		let r = self.widget(id);
		if active {
			self.fill(r, BTN_ACTIVE);
		}
		self.image_rect_center(icon, r);
		self.is_click()
	}

	fn btn_mini<F: FnMut()>(&mut self, id: WidgetId, label: &str, mut cb: F) {
		let r = self.widget(id);
		if self.is_hot() && self.is_active() {
			self.fill(r, BTN_ACTIVE);
		};
		self.text_center(r, LABEL_COLOR, label);
		if self.is_click() {
			cb();
		}
	}

	fn switch<T, A, B, C>(&self, normal: A, hot: B, active: C) -> Option<T>
		where
			A: Into<Option<T>>,
			B: Into<Option<T>>,
			C: Into<Option<T>>,
	{
		if !self.is_hot() {
			normal.into()
		} else if self.is_active() {
			active.into()
		} else {
			hot.into()
		}
	}

	fn btn_label_left(&mut self, id: WidgetId, label: &str) -> bool {
		let r = self.widget(id);
		//if let Some(bg) = self.btn_bg(None, Some(BTN_BG), Some(BTN_ACTIVE)) {
		if let Some(bg) = self.switch(None, None, BTN_ACTIVE) {
			self.fill(r, bg);
		}
		self.border(r, BTN_BORDER);
		self.text_center_left(r.inset_x(INSET_X), LABEL_COLOR, label);
		self.is_click()
	}

	fn btn_label(&mut self, id: WidgetId, label: &str) -> bool {
		let r = self.widget(id);

		let bg = 0x353D4B_FF;
		let active_color = 0x0076FF_FF;

		if let Some(bg) = self.switch(None, bg, active_color) {
			self.fill(r, bg);
		}
		self.border(r, bg);
		self.text_center(r, LABEL_COLOR, label);
		self.is_click()
	}

	fn checkbox_cell(&mut self, id: WidgetId, value: &Cell<bool>) -> bool {
		let r = self.widget(id);
		let click = self.is_click();
		if click {
			value.set(!value.get());
		}
		self.draw_checkbox(r, value.get());
		click
	}

	fn checkbox_label(&mut self, id: WidgetId, label: &str, value: &mut bool) {
		let r = self.widget(id);
		if self.is_click() {
			*value = !*value;
		}
		let check = r.w(r.dy());
		let lab = r.x(r.dy() + INSET_X);
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