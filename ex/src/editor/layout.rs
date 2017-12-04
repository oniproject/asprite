use super::theme::*;

use ::graphics;

use math::*;
use ui::*;

use std::mem::replace;

pub struct EditorLayout<'a, 'state> {
	state: &'state mut UiState,
	ctx: Context<'a, graphics::Graphics>,
	cursor: Vector2<f32>,
	indent: usize,

	update: bool,
}

impl<'a, 'state> EditorLayout<'a, 'state> {
	pub fn new(ctx: Context<'a, graphics::Graphics>, state: &'state mut UiState) -> Self {
		Self {
			ctx: ctx.sub_range(0xFFFF),
			state,
			cursor: Vector2::zero(),
			indent: 0,
			update: false,
		}
	}

	pub fn check_reset(&mut self) -> bool {
		replace(&mut self.update, false)
	}

	pub fn indent(&self) -> usize { self.indent }
	pub fn incr_indent(&mut self) {
		self.indent += 1;
		self.cursor.x = self.indent as f32 * 10.0;
	}
	pub fn decr_indent(&mut self) {
		self.indent -= 1;
		self.cursor.x = self.indent as f32 * 10.0;
	}

	fn one_line(&mut self) -> Context<'a, graphics::Graphics> {
		let rect = self.ctx.rect().pad_x(8.0);

		let min = rect.min + self.cursor;
		let max = Point2::new(rect.max.x, min.y + 20.0);

		self.cursor.y += 20.0;

		let rect = Rect { min, max };
		self.ctx.sub_rect(rect)
	}

	pub fn tree<F>(&mut self, label: &str, cb: F)
		where F: FnOnce(&mut Self)
	{
		self.label(label);
		self.incr_indent();
		cb(self);
		self.decr_indent();
	}

	pub fn label(&mut self, label: &str) {
		let ctx = self.one_line();
		ctx.label(0.0, 0.5, [0xFF;4], label);
	}

	fn one_line_prop(&mut self, label: &str) -> Context<'a, graphics::Graphics> {
		let (label_ctx, ret) = self.one_line().split_x(0.3);
		label_ctx.label(0.0, 0.5, [0xFF;4], label);
		ret
	}

	pub fn angle(&mut self, label: &str, angle: &mut f32) {
		let ctx = self.one_line_prop(label);

		let mut slider = SliderModel {
			min: -f32::PI,
			max: f32::PI,
			// because [-pi, pi] vs [-pi, pi)
			current: if *angle > f32::PI || *angle < -f32::PI {
				angle.normalize_angle(0.0)
			} else { *angle },
		};

		let rect = ctx.rect();
		let pad = (rect.dy() - 2.0) / 2.0;
		ctx.quad([0xAA; 4], &rect.pad_y(pad));

		HSLIDER.behavior(&ctx, &mut self.state, &mut slider);

		let start = *angle;
		*angle = slider.current;
		self.update |= *angle == start;
	}

	pub fn vector2(&mut self, label: &str, v: &mut Vector2<f32>, add: f32) {
		let ctx = self.one_line_prop(label);
		let (x, y) = ctx.split_x(0.5);
		self.update |= edit_f(&x, &mut self.state, &mut v.x, "X", add);
		self.update |= edit_f(&y, &mut self.state, &mut v.y, "Y", add);
	}
}

pub fn edit_f(ctx: &Context<graphics::Graphics>, state: &mut UiState, v: &mut f32, label: &str, scale: f32) -> bool {
	let wh = ctx.rect().dy();
	let widgets = [
		Flow::with_wh(wh, wh), // base label
		Flow::with_wh(wh, wh), // -
		Flow::with_wh(wh, wh).along_weight(1.0).expand_along(),
		Flow::with_wh(wh, wh), // +
	];

	let mut flag = false;

	let mut iter = ctx.horizontal_flow(0.0, 0.0, &widgets);

	iter.next().unwrap().label(0.5, 0.5, [0xFF;4], label);

	let sub = &iter.next().unwrap();
	let value = &iter.next().unwrap();
	let add = &iter.next().unwrap();

	if BTN.behavior(add, state, &mut ()) {
		*v += scale;
		flag = true;
	}

	if BTN.behavior(sub, state, &mut ()) {
		*v -= scale;
		flag = true;
	}

	add.label(0.5, 0.5, [0xFF;4], "+");
	sub.label(0.5, 0.5, [0xFF;4], "-");
	value.label(0.5, 0.5, [0xFF;4], &format!("{}", *v));

	flag
}
