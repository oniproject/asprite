use super::*;
use math::*;

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
			ctx.set_hovered();

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
			ctx.set_hovered();

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

