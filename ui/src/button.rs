use super::*;
use math::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	Normal,
	Hovered,
	Pressed,
	Disabled,
}

fn btn_behavior<F, D>(ctx: &Context<D>, state: &mut UiState, hovered: bool, interactable: bool, callback: F) -> Mode
	where F: FnOnce(), D: ?Sized + Graphics
{
	let id = ctx.reserve_widget_id();
	let active = state.is_active(id);
	match (interactable, hovered) {
		(true, false) => Mode::Normal,
		(true, true) if active => {
			if ctx.was_released() {
				state.active_widget = None;
				callback()
			}
			Mode::Pressed
		}
		(true, true) => {
			if ctx.was_pressed() {
				state.active_widget = Some(id);
			}
			Mode::Hovered
		}
		(false, _) => Mode::Disabled,
	}
}

pub trait ButtonStyle<D: ?Sized + Graphics> {
	fn draw_button(&self, Mode, &D, &Rect<f32>);
}

pub struct ToggleStyle<'a, D: ?Sized + Graphics + 'a> {
	pub checked: &'a ButtonStyle<D>,
	pub unchecked: &'a ButtonStyle<D>,
}

impl<'a, D: ?Sized + Graphics + 'a> Toggle<D> for ToggleStyle<'a, D> {
	fn draw_toggle(&self, toggle: bool, mode: Mode, draw: &D, rect: &Rect<f32>) {
		if toggle {
			self.checked.draw_button(mode, draw, rect);
		} else {
			self.unchecked.draw_button(mode, draw, rect);
		};
	}
}

pub trait Toggle<D: ?Sized + Graphics> {
	fn draw_toggle(&self, bool, Mode, &D, &Rect<f32>);
	fn toggle(&self, ctx: &Context<D>, state: &mut UiState, toggle: &mut bool, interactable: bool) {
		let hovered = ctx.is_cursor_hovering();
		let mode = btn_behavior(ctx, state, hovered, interactable, || *toggle = !*toggle);
		self.draw_toggle(*toggle, mode, &ctx.draw(), &ctx.rect());
	}
}

impl<T, D: ?Sized + Graphics> Button<D> for T where T: ButtonStyle<D> {}

pub trait Button<D: ?Sized + Graphics>: ButtonStyle<D> {
	fn run(&self, ctx: &Context<D>, state: &mut UiState, interactable: bool) -> bool {
		let hovered = ctx.is_cursor_hovering();
		let mut ret = false;
		let mode = btn_behavior(ctx, state, hovered, interactable, || ret = true);
		self.draw_button(mode, &ctx.draw(), &ctx.rect());
		ret
	}
}

pub struct ImageButton<'a, D: ?Sized + Graphics + 'a> {
	pub normal: &'a D::Texture,
	pub hovered: &'a D::Texture,
	pub pressed: &'a D::Texture,
	pub disabled: &'a D::Texture,
}

impl<'a, D: ?Sized + Graphics + 'a> ButtonStyle<D> for ImageButton<'a, D> {
	fn draw_button(&self, mode: Mode, draw: &D, rect: &Rect<f32>) {
		let texture = match mode {
			Mode::Normal   => self.normal,
			Mode::Hovered  => self.hovered,
			Mode::Disabled => self.disabled,
			Mode::Pressed  => self.pressed,
		};
		draw.texture(texture, rect);
	}
}

pub struct ColorButton<D: ?Sized + Graphics> {
	pub normal: D::Color,
	pub hovered: D::Color,
	pub pressed: D::Color,
	pub disabled: D::Color,
}

impl<D: ?Sized + Graphics> ButtonStyle<D> for ColorButton<D> {
	fn draw_button(&self, mode: Mode, draw: &D, rect: &Rect<f32>) {
		let color = match mode {
			Mode::Normal   => self.normal,
			Mode::Hovered  => self.hovered,
			Mode::Disabled => self.disabled,
			Mode::Pressed  => self.pressed,
		};
		draw.quad(color, rect);
	}
}
