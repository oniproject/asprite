use super::theme::*;

use ::graphics;

use math::*;
use ui::*;

pub struct EditorLayout<'a, 'state> {
	state: &'state mut UiState,
	ctx: Context<'a, graphics::Graphics>,
	cursor: Vector2<f32>,
	indent: usize,
}

impl<'a, 'state> EditorLayout<'a, 'state> {
	pub fn new(ctx: Context<'a, graphics::Graphics>, state: &'state mut UiState) -> Self {
		Self {
			ctx, state,
			cursor: Vector2::zero(),
			indent: 0,
		}
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
		self.ctx.sub().with_rect(rect).build()
	}

	pub fn label(&mut self, label: &str) {
		let ctx = self.one_line();
		ctx.label(0.0, 0.5, [0xFF;4], label);
	}

	pub fn angle(&mut self, label: &str, a: &mut f32) -> bool {
		let ctx = self.one_line();

		let (lb, val) = ctx.split_x(0.3);
		lb.label(0.0, 0.5, [0xFF;4], label);

		let start = *a;

		let mut slider = SliderModel {
			min: -f32::PI,
			max: f32::PI,
			current: start.normalize_angle(0.0),
		};

		HSLIDER.behavior(&val, &mut self.state, &mut slider);

		*a = slider.current;

		*a == start
	}

	pub fn vector2(&mut self, label: &str, v: &mut Vector2<f32>, add: f32) -> bool {
		let ctx = self.one_line();

		let (lb, val) = ctx.split_x(0.3);
		lb.label(0.0, 0.5, [0xFF;4], label);

		let (x, y) = val.split_x(0.5);
		let x = edit_f(&x, &mut self.state, &mut v.x, "X", add);
		let y = edit_f(&y, &mut self.state, &mut v.y, "Y", add);
		x || y
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
