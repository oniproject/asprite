#![allow(dead_code)]

mod guard;

mod id;
mod transform;
mod flow;
mod context;
mod graphics;
mod mouse;
mod button;
mod ninepatch;

pub use self::transform::*;

pub use self::flow::*;
pub use self::guard::*;
pub use self::context::*;
pub use self::id::*;
pub use self::graphics::*;
pub use self::mouse::*;
pub use self::button::*;
pub use self::ninepatch::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiState {
	pub active_widget: Option<Id>,
}

impl UiState {
	pub const fn new() -> Self {
		Self { active_widget: None }
	}
	pub fn is_active(&self, id: Id) -> bool {
		self.active_widget == Some(id)
	}
}
