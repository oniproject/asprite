use std::any::Any;
use std::marker::PhantomData;
use specs::{self, Entity, Entities, Join, WriteStorage};
use specs::{Component, DenseVecStorage, FlaggedStorage};
use specs::UnprotectedStorage;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use hibitset::BitSet;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Parent(pub Entity);

impl Component for Parent {
	type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

pub trait Transform {
	type Local;
	type Global;

	fn convert(&Self::Local) -> Self::Global;
	fn combine(&Self::Global, &Self::Global) -> Self::Global;
	fn rewrite(&mut Self::Global, &Self::Global);
}

/// Handles updating `GlobalTransform` components based
/// on the `LocalTransform` component and parents.
#[derive(Derivative)]
#[derivative(Default)]
pub struct System<T> {
	/// Map of entities to index in sorted vec.
	indices: HashMap<Entity, usize>,
	/// Vec of entities with parents before children.
	/// Only contains entities with parents.
	sorted: Vec<Entity>,

	init: BitSet,
	frame_init: BitSet,

	dead: HashSet<Entity>,
	remove_parent: Vec<Entity>,

	#[derivative(Default(value="PhantomData"))] _t: PhantomData<T>,
}

impl<T> System<T> {
	fn remove(&mut self, index: usize) {
		let entity = self.sorted[index];
		self.sorted.swap_remove(index);
		if let Some(swapped) = self.sorted.get(index) {
			self.indices.insert(*swapped, index);
		}
		self.indices.remove(&entity);
		self.init.remove(index as u32);
	}
}

impl<'a, T, L, G, LS, GS> specs::System<'a> for System<T>
	where
		T: Transform<Local=L, Global=G>,
		L: Component<Storage=FlaggedStorage<L, LS>> + Send + Sync,
		G: Component<Storage=FlaggedStorage<G, GS>> + Send + Sync,
		LS: UnprotectedStorage<L> + Any + Send + Sync,
		GS: UnprotectedStorage<G> + Any + Send + Sync,
{
	type SystemData = (
		Entities<'a>,
		WriteStorage<'a, L>,
		WriteStorage<'a, Parent>,
		WriteStorage<'a, G>,
	);
	fn run(&mut self, (entities, mut locals, mut parents, mut globals): Self::SystemData) {
		#[cfg(feature = "profiler")] profile_scope!("transform_system");

		// Clear dirty flags on `Transform` storage, before updates go in
		(&mut globals).open().1.clear_flags();

		{
			for (entity, parent) in (&*entities, parents.open().1).join() {
				if parent.0 == entity {
					self.remove_parent.push(entity);
				}
			}
			for entity in self.remove_parent.iter() {
				eprintln!("Entity was its own parent: {:?}", entity);
				parents.remove(*entity);
			}
			self.remove_parent.clear();
		}

		{
			// Checks for entities with a modified local transform
			// or a modified parent, but isn't initialized yet.

			// has a local, parent, and isn't initialized.
			let filter = locals.open().0 & parents.open().0 & !&self.init;
			for (entity, _) in (&*entities, &filter).join() {
				self.indices.insert(entity, self.sorted.len());
				self.sorted.push(entity);
				self.frame_init.add(entity.id());
			}
		}

		{
			let locals_flagged = locals.open().1;

			// Compute transforms without parents.
			for (_entity, local, global, _) in
				(&*entities, locals_flagged, &mut globals, !&parents).join()
			{
				T::rewrite(global, &T::convert(local));
			}
		}

		// Compute transforms with parents.
		let mut index = 0;
		while index < self.sorted.len() {
			let entity = self.sorted[index];
			let local_dirty = locals.open().1.flagged(entity);
			let parent_dirty = parents.open().1.flagged(entity);

			match (
				parents.get(entity),
				locals.get(entity),
				self.dead.contains(&entity),
			) {
				(Some(parent), Some(local), false) => {
					// Make sure this iteration isn't a child before the parent.
					if parent_dirty {
						let mut swap = None;

						// If the index is none then the parent is an orphan or dead
						if let Some(parent_index) = self.indices.get(&parent.0) {
							if parent_index > &index {
								swap = Some(*parent_index);
							}
						}

						if let Some(p) = swap {
							// Swap the parent and child.
							self.sorted.swap(p, index);
							self.indices.insert(parent.0, index);
							self.indices.insert(entity, p);

							// Swap took place, re-try this index.
							continue;
						}
					}

					// Kill the entity if the parent is dead.
					if self.dead.contains(&parent.0) || !entities.is_alive(parent.0) {
						self.remove(index);
						let _ = entities.delete(entity);
						self.dead.insert(entity);

						// Re-try index because swapped with last element.
						continue;
					}

					if local_dirty || parent_dirty || globals.open().1.flagged(parent.0) {
						let combined = T::convert(local);
						let combined = match globals.get(parent.0) {
							Some(g) => T::combine(g, &combined),
							None => combined,
						};
						if let Some(global) = globals.get_mut(entity) {
							T::rewrite(global, &combined);
						}
					}
				}
				(_, _, dead @ _) => {
					// This entity should not be in the sorted list, so remove it.
					self.remove(index);

					if !dead && !entities.is_alive(entity) {
						self.dead.insert(entity);
					}

					// Re-try index because swapped with last element.
					continue;
				}
			}

			index += 1;
		}

		(&mut locals).open().1.clear_flags();
		(&mut parents).open().1.clear_flags();

		for bit in &self.frame_init {
			self.init.add(bit);
		}
		self.frame_init.clear();
		self.dead.clear();
	}
}

#[cfg(test)]
mod tests {
	use specs::{
		World, RunNow,
		Component, DenseVecStorage, FlaggedStorage,
	};

	use super::Parent;

	#[derive(Debug, Copy, Clone, PartialEq)]
	struct Global(usize);
	#[derive(Debug, Copy, Clone)]
	pub struct Local(u8);

	impl Component for Global {
		type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
	}
	impl Component for Local {
		type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
	}

	struct Transform;
	impl super::Transform for Transform {
		type Local = Local;
		type Global = Global;
		#[inline]
		fn convert(l: &Local) -> Global {
			Global(l.0 as usize)
		}
		#[inline]
		fn combine(a: &Global, b: &Global) -> Global {
			Global(a.0 + b.0)
		}
		#[inline]
		fn rewrite(dst: &mut Global, src: &Global) {
			dst.0 = src.0
		}
	}

	type System = super::System<Transform>;

	fn transform_world<'a, 'b>() -> (World, System) {
		let mut world = World::new();
		world.register::<Local>();
		world.register::<Global>();
		world.register::<Parent>();

		(world, System::default())
	}

	#[test]
	fn basic() {
		let (mut world, mut system) = transform_world();

		let e = world
			.create_entity()
			.with(Local(5))
			.with(Global(0))
			.build();

		system.run_now(&mut world.res);

		let gt = world.read::<Global>().get(e).unwrap().clone();
		assert_eq!(gt, Global(5));
	}

	#[test]
	fn entity_is_parent() {
		let (mut world, mut system) = transform_world();

		let e = world
			.create_entity()
			.with(Local(0))
			.with(Global(0))
			.build();

		world.write::<Parent>().insert(e, Parent(e));
		system.run_now(&mut world.res);

		let parents = world.read::<Parent>();
		assert_eq!(parents.get(e), None)
	}

	#[test]
	fn parent_before() {
		let (mut world, mut system) = transform_world();

		let e1 = world.create_entity()
			.with(Local(1))
			.with(Global(0))
			.build();

		let e2 = world.create_entity()
			.with(Local(2))
			.with(Global(0))
			.with(Parent(e1))
			.build();

		let e3 = world.create_entity()
			.with(Local(3))
			.with(Global(0))
			.with(Parent(e2))
			.build();

		system.run_now(&mut world.res);

		let gt = world.read::<Global>();
		assert_eq!(gt.get(e1).unwrap().clone(), Global(1));
		assert_eq!(gt.get(e2).unwrap().clone(), Global(3));
		assert_eq!(gt.get(e3).unwrap().clone(), Global(6));
	}

	#[test]
	fn parent_after() {
		let (mut world, mut system) = transform_world();

		let e1 = world.create_entity()
			.with(Local(1))
			.with(Global(0))
			.build();

		let e2 = world.create_entity()
			.with(Local(2))
			.with(Global(0))
			.build();

		let e3 = world.create_entity()
			.with(Local(3))
			.with(Global(0))
			.build();

		{
			let mut parents = world.write::<Parent>();
			parents.insert(e2, Parent(e1));
			parents.insert(e3, Parent(e2));
		}

		system.run_now(&mut world.res);

		let gt = world.read::<Global>();
		assert_eq!(gt.get(e1).unwrap().clone(), Global(1));
		assert_eq!(gt.get(e2).unwrap().clone(), Global(3));
		assert_eq!(gt.get(e3).unwrap().clone(), Global(6));
	}

	#[test]
	fn parent_removed() {
		let (mut world, mut system) = transform_world();

		let e1 = world.create_entity()
			.with(Local(0))
			.with(Global(0))
			.build();

		let e2 = world.create_entity()
			.with(Local(0))
			.with(Global(0))
			.with(Parent(e1))
			.build();

		let e3 = world.create_entity()
			.with(Local(0))
			.with(Global(0))
			.build();

		let e4 = world.create_entity()
			.with(Local(0))
			.with(Global(0))
			.with(Parent(e3))
			.build();

		let e5 = world.create_entity()
			.with(Local(0))
			.with(Global(0))
			.with(Parent(e4))
			.build();

		let _ = world.delete_entity(e1);
		system.run_now(&mut world.res);
		world.maintain();

		assert_eq!(world.is_alive(e1), false);
		assert_eq!(world.is_alive(e2), false);

		let _ = world.delete_entity(e3);
		system.run_now(&mut world.res);
		world.maintain();

		assert_eq!(world.is_alive(e3), false);
		assert_eq!(world.is_alive(e4), false);
		assert_eq!(world.is_alive(e5), false);
	}
}
