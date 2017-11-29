use super::*;
use math::*;

pub type SimpleButton<D> = Button<D, D, D>;
pub type ColorButton<D> = SimpleButton<ColorDrawer<D>>;
pub type TextureButton<D> = SimpleButton<TextureDrawer<D>>;

pub trait Component<D: ?Sized + Graphics> {
	type Event;
	type Model;
	fn behavior(&self, ctx: &Context<D>, state: &mut UiState, model: &mut Self::Model) -> Self::Event;
}

pub struct Button<P, H, N> {
	pub pressed: P,
	pub hovered: H,
	pub normal: N,
}

impl<D, P, H, N> Component<D> for Button<P, H, N>
	where
		D: ?Sized + Graphics,
		P: FrameDrawer<D>,
		H: FrameDrawer<D>,
		N: FrameDrawer<D>,
{
	type Event = bool;
	type Model = ();
	fn behavior(&self, ctx: &Context<D>, state: &mut UiState, _: &mut Self::Model) -> Self::Event {
		let id = ctx.reserve_widget_id();
		if state.active_widget == Some(id) {
			self.pressed.draw_frame(ctx.draw(), ctx.rect());
			let event = ctx.was_released();
			if event || !ctx.is_cursor_hovering() {
				state.active_widget = None;
			}
			event
		} else if ctx.is_cursor_hovering() && state.active_widget == None {
			self.hovered.draw_frame(ctx.draw(), ctx.rect());
			if ctx.was_pressed() {
				state.active_widget = Some(id);
			}
			false
		} else {
			self.normal.draw_frame(ctx.draw(), ctx.rect());
			false
		}
	}
}

pub struct Toggle<P, H, N> {
	pub checked: Button<P, H, N>,
	pub unchecked: Button<P, H, N>,
}

impl<D, P, H, N> Component<D> for Toggle<P, H, N>
	where
		D: ?Sized + Graphics,
		P: FrameDrawer<D>,
		H: FrameDrawer<D>,
		N: FrameDrawer<D>,
{
	type Event = ();
	type Model = bool;
	fn behavior(&self, ctx: &Context<D>, state: &mut UiState, checked: &mut Self::Model) -> Self::Event {
		if *checked {
			if self.checked.behavior(ctx, state, &mut ()) {
				*checked = false;
			}
		} else {
			if self.unchecked.behavior(ctx, state, &mut ()) {
				*checked = true;
			}
		}
	}
}

fn lerp(min: f32, max: f32, t: f32) -> f32 {
	(1.0 - t) * min + t * max
}
fn clamp01(v: f32) -> f32 {
	if v < 0.0 {
		0.0
	} else if v > 1.0 {
		1.0
	} else {
		v
	}
}

pub struct SliderModel {
	pub current: f32,
	pub min: f32,
	pub max: f32,
	pub vertical: bool,
}

impl SliderModel {
	fn set_percent(&mut self, value: f32) {
		self.current = lerp(self.min, self.max, value);
	}
	fn percent(&mut self) -> f32 {
		clamp01((self.current - self.min) / (self.max - self.min))
	}
}

pub struct Slider<BG, F, H> {
	pub progress: Progress<BG, F>,
	pub pressed_handle: H,
	pub hovered_handle: H,
	pub normal_handle: H,
}

impl<D, BG, F, H> Component<D> for Slider<BG, F, H>
	where
		D: ?Sized + Graphics,
		BG: FrameDrawer<D>,
		F: FrameDrawer<D>,
		H: FrameDrawer<D>,
{
	type Event = ();
	type Model = SliderModel;
	fn behavior(&self, ctx: &Context<D>, state: &mut UiState, slide: &mut Self::Model) -> Self::Event {
		let id = ctx.reserve_widget_id();
		let hovered = ctx.is_cursor_hovering();

		let rect = ctx.rect();

		let handle = if state.active_widget == Some(id) {
			let (pos, min, delta) = if slide.vertical {
				(ctx.mouse().cursor.y, rect.min.y, rect.dy())
			} else {
				(ctx.mouse().cursor.x, rect.min.x, rect.dx())
			};

			slide.set_percent(clamp01((pos - min) / delta));

			if ctx.was_released() {
				state.active_widget = None;
			}
			&self.pressed_handle
		} else if hovered && state.active_widget.is_none() {
			if ctx.was_pressed() {
				state.active_widget = Some(id);
			}
			&self.hovered_handle
		} else {
			&self.normal_handle
		};

		let percent = slide.percent();
		let handle_rect = if slide.vertical {
			let delta = rect.dx() / 2.0;
			let y = lerp(rect.min.y + delta, rect.max.y - delta, percent);
			let p = Point2::new(rect.min.x + delta, y);
			(Rect { min: p, max: p }).inset(-delta)
		} else {
			let delta = rect.dy() / 2.0;
			let x = lerp(rect.min.x + delta, rect.max.x - delta, percent);
			let p = Point2::new(x, rect.min.y + delta);
			(Rect { min: p, max: p }).inset(-delta)
		};

		self.progress.behavior(ctx, state, &mut (percent, slide.vertical));
		handle.draw_frame(ctx.draw(), handle_rect);
	}
}

pub struct Progress<BG, F> {
	pub background: BG,
	pub fill: F,
}

impl<D, BG, F> Component<D> for Progress<BG, F>
	where
		D: ?Sized + Graphics,
		BG: FrameDrawer<D>,
		F: FrameDrawer<D>,
{
	type Event = ();
	type Model = (f32, bool);
	fn behavior(&self, ctx: &Context<D>, _state: &mut UiState, model: &mut Self::Model) -> Self::Event {
		let rect = ctx.rect();
		let fill_rect = if model.1 {
			rect.h(rect.dy() * model.0)
		} else {
			rect.w(rect.dx() * model.0)
		};
		self.background.draw_frame(ctx.draw(), rect);
		self.fill.draw_frame(ctx.draw(), fill_rect);
	}
}
