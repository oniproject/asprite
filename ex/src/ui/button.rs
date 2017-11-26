use super::*;
use math::*;

pub trait Button<D: ?Sized + Graphics> {
	fn normal(&self, &D, &Rect<f32>);
	fn hovered(&self, &D, &Rect<f32>);
	fn active(&self, &D, &Rect<f32>);
	fn disabled(&self, &D, &Rect<f32>);

	fn run(&self, ctx: &Context<D>, state: &mut UiState, interactable: bool) -> bool {
		let id = ctx.reserve_widget_id();

		if interactable {
			if ctx.is_cursor_hovering() {
				ctx.set_hovered_widget(id);
				if state.is_active(id) {
					self.active(&ctx.draw(), &ctx.rect());
					if ctx.was_released() {
						state.active_widget = None;
						return true;
					}
				} else {
					self.hovered(&ctx.draw(), &ctx.rect());
					if ctx.was_pressed() {
						state.active_widget = Some(id);
					}
				}
			} else {
				self.normal(&ctx.draw(), &ctx.rect());
			}
		} else {
			self.disabled(&ctx.draw(), &ctx.rect());
		}
		false
	}
}

pub struct ImageButton<'a, D: ?Sized + Graphics + 'a> {
	pub normal: &'a D::Texture,
	pub hovered: &'a D::Texture,
	pub active: &'a D::Texture,
	pub disabled: &'a D::Texture,
}

impl<'a, D: ?Sized + Graphics + 'a> Button<D> for ImageButton<'a, D> {
	#[inline]
	fn normal(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.normal, rect);
	}
	#[inline]
	fn hovered(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.hovered, rect);
	}
	#[inline]
	fn active(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.active, rect);
	}
	#[inline]
	fn disabled(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.disabled, rect);
	}
}

pub struct ColorButton<D: ?Sized + Graphics> {
	pub normal: D::Color,
	pub hovered: D::Color,
	pub active: D::Color,
	pub disabled: D::Color,
}

impl<D: ?Sized + Graphics> Button<D> for ColorButton<D> {
	#[inline]
	fn normal(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.normal, rect);
	}
	#[inline]
	fn hovered(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.hovered, rect);
	}
	#[inline]
	fn active(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.active, rect);
	}
	#[inline]
	fn disabled(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.disabled, rect);
	}
}
