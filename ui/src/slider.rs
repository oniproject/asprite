/*
pub trait Base {
	fn is_active(&self, id: Id) -> bool;
	fn active_widget(&self) -> Option<Id>;
	fn set_active_widget(&mut self, Option<Id>);
}

pub trait Widget {
	type Event;
	type Style;
	type Model: BaseState;
}

struct Button;

impl Widget for Btn {
	type Event = ();

	fn run(ctx: &Context<D>, state: &mut UiState, model: &Self::Model) -> Option<Self::Event> {
	}

	fn draw(style: &Self::Style) {
	}
}

pub enum Direction {
	LeftToRight,
	RightToLeft,
	BottomToTop,
	TopToBottom,
}

struct Data {
	current: f32,
	min: f32,
	max: f32,
}

struct Handle {
}

struct Fill {
}

pub enum Mode {
	Normal,
	Hovered,
	Active,
}
*/

// horizontal
pub fn slider<F, D>(ctx: &Context<D>, state: &mut UiState, mut current: f32) -> f32 {
}

/*
pub fn slider_behavior(dir: Direction, min: f32, max: f32, current: &mut f32) {
	let id = ctx.reserve_widget_id();
	let active = state.is_active(id);
	let hovered = hovered && (active || state.active_widget.is_none());
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
*/
