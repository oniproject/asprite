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
	fn normal(&self, &D, &Rect<f32>);
	fn hovered(&self, &D, &Rect<f32>);
	fn pressed(&self, &D, &Rect<f32>);
	fn disabled(&self, &D, &Rect<f32>);
}

pub struct ToggleStyle<'a, D: ?Sized + Graphics + 'a> {
	pub checked: &'a ButtonStyle<D>,
	pub unchecked: &'a ButtonStyle<D>,
}

impl<'a, D: ?Sized + Graphics + 'a> ToggleStyle<'a, D> {
	fn style(&self, toggle: bool) -> &'a ButtonStyle<D> {
		if toggle {
			self.checked
		} else {
			self.unchecked
		}
	}
}

impl<'a, D: ?Sized + Graphics + 'a> Toggle<D> for ToggleStyle<'a, D> {
	fn normal(&self, toggle: bool, draw: &D, rect: &Rect<f32>) {
		self.style(toggle).normal(draw, rect);
	}
	fn pressed(&self, toggle: bool, draw: &D, rect: &Rect<f32>) {
		self.style(toggle).pressed(draw, rect);
	}
	fn hovered(&self, toggle: bool, draw: &D, rect: &Rect<f32>) {
		self.style(toggle).hovered(draw, rect);
	}
	fn disabled(&self, toggle: bool, draw: &D, rect: &Rect<f32>) {
		self.style(toggle).disabled(draw, rect);
	}
}

pub trait Toggle<D: ?Sized + Graphics> {
	fn normal(&self, bool, &D, &Rect<f32>);
	fn pressed(&self, bool, &D, &Rect<f32>);
	fn hovered(&self, bool, &D, &Rect<f32>);
	fn disabled(&self, bool, &D, &Rect<f32>);

	fn toggle(&self, ctx: &Context<D>, state: &mut UiState, toggle: &mut bool, interactable: bool) {
		let hovered = ctx.is_cursor_hovering();
		let mode = btn_behavior(ctx, state, hovered, interactable, || *toggle = !*toggle);
		match mode {
			Mode::Normal   => self.normal(  *toggle, &ctx.draw(), &ctx.rect()),
			Mode::Hovered  => self.hovered( *toggle, &ctx.draw(), &ctx.rect()),
			Mode::Disabled => self.disabled(*toggle, &ctx.draw(), &ctx.rect()),
			Mode::Pressed  => self.pressed( *toggle, &ctx.draw(), &ctx.rect()),
		}
	}
}

impl<T, D: ?Sized + Graphics> Button<D> for T where T: ButtonStyle<D> {}

pub trait Button<D: ?Sized + Graphics>: ButtonStyle<D> {
	fn run(&self, ctx: &Context<D>, state: &mut UiState, interactable: bool) -> bool {
		let hovered = ctx.is_cursor_hovering();
		let mut ret = false;
		let mode = btn_behavior(ctx, state, hovered, interactable, || ret = true);
		match mode {
			Mode::Normal   => self.normal(  &ctx.draw(), &ctx.rect()),
			Mode::Hovered  => self.hovered( &ctx.draw(), &ctx.rect()),
			Mode::Disabled => self.disabled(&ctx.draw(), &ctx.rect()),
			Mode::Pressed  => self.pressed( &ctx.draw(), &ctx.rect()),
		}
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
	#[inline]
	fn normal(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.normal, rect);
	}
	#[inline]
	fn hovered(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.hovered, rect);
	}
	#[inline]
	fn pressed(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.pressed, rect);
	}
	#[inline]
	fn disabled(&self, draw: &D, rect: &Rect<f32>) {
		draw.texture(self.disabled, rect);
	}
}

pub struct ColorButton<D: ?Sized + Graphics> {
	pub normal: D::Color,
	pub hovered: D::Color,
	pub pressed: D::Color,
	pub disabled: D::Color,
}

impl<D: ?Sized + Graphics> ButtonStyle<D> for ColorButton<D> {
	#[inline]
	fn normal(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.normal, rect);
	}
	#[inline]
	fn hovered(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.hovered, rect);
	}
	#[inline]
	fn pressed(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.pressed, rect);
	}
	#[inline]
	fn disabled(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.disabled, rect);
	}
}

/*
pub struct ColorToggle<D: ?Sized + Graphics> {
	pub normal: D::Color,
	pub hovered: D::Color,
	pub pressed: D::Color,
	pub disabled: D::Color,
}

impl<D: ?Sized + Graphics> ButtonStyle<D> for ColorButton<D> {
	#[inline]
	fn normal(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.normal, rect);
	}
	#[inline]
	fn hovered(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.hovered, rect);
	}
	#[inline]
	fn pressed(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.pressed, rect);
	}
	#[inline]
	fn disabled(&self, draw: &D, rect: &Rect<f32>) {
		draw.quad(self.disabled, rect);
	}
}
*/
