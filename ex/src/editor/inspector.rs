use super::layout::*;
use super::theme::*;

use ui::*;
use math::*;
use specs::{
	self,
	Entity, World, Join,
	FlaggedStorage,
	UnprotectedStorage,
};
use graphics;
use std::any::Any;

use sprite::{Local, Sprite};

pub struct Inspector {
	entity: Option<Entity>,
	//editors: Vec<(::shred::ResourceId, ())>,
	/*
	 * resources ids list
	 *
	*/
}

impl Inspector {
	pub fn new() -> Self {
		Self { entity: None }
	}

	pub fn set_entity(&mut self, entity: Option<Entity>) {
		self.entity = entity;
	}

	pub fn run(&self, ctx: &Context<graphics::Graphics>, state: &mut UiState, world: &mut World) {
		let rect = ctx.rect();
		let body = rect.pad_min_y(BAR_TITLE_HEIGHT);

		let header = Rect {
			min: rect.min,
			max: Point2::new(rect.max.x, rect.min.y + BAR_TITLE_HEIGHT),
		};

		let header = ctx.sub_rect(header);
		let body = ctx.sub_rect(body);

		header.quad(BAR_TITLE_BG, &header.rect());
		body.quad(BAR_BG, &body.rect());

		if let Some(e) = self.entity {
			header.label(0.5, 0.5, [0xFF; 4], &format!("Inspector: {:?}", e));

			let mut lay = EditorLayout::new(body, state);

			component_flagged(world, e, |t: &mut Local| {
				lay.tree("Transform", |lay| {
					lay.vector2("Position", &mut t.position, 5.0);
					lay.angle("Rotation", &mut t.rotation);
					lay.vector2("Scale", &mut t.scale, 0.1);
					lay.vector2("Skew", &mut t.skew, 0.1);
					lay.vector2("Pivot", &mut t.pivot, 1.0);
				});
				lay.check_reset()
			});

			component(world, e, |t: &mut Sprite| {
				lay.tree("Sprite", |lay| {
					lay.vector2("Anchor", &mut t.anchor, 0.1);
				});
				lay.check_reset()
			});

		} else {
			header.label(0.5, 0.5, [0xFF; 4], "Inspector: None");
			body.label(0.5, 0.5, [0xCC; 4], "Please select entity");
		}
	}
}

pub fn component<'a, C, S, F>(world: &mut World, e: Entity, f: F) -> bool
	where
		F: FnOnce(&mut C) -> bool,
		C: specs::Component<Storage=S> + Send + Sync,
		S: UnprotectedStorage<C> + Any + Send + Sync,
{
	let mut store = world.write::<C>();
	store.get_mut(e).map(f).unwrap_or(false)
}

pub fn component_flagged<'a, C, S, F>(world: &mut World, e: Entity, f: F)
	where
		F: FnOnce(&mut C) -> bool,
		C: specs::Component<Storage=FlaggedStorage<C, S>> + Send + Sync,
		S: UnprotectedStorage<C> + Any + Send + Sync,
{
	let mut store = world.write::<C>();
	if store.get_mut(e).map(f).unwrap_or(false) {
		(&mut store).open().1.flag(e);
	}
}
