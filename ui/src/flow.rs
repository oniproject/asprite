use super::*;
use std::cell::{Ref, RefMut, RefCell, Cell};
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum Axis {
	Horizontal,
	Vertical,
}

pub struct FlowData<N> {
	pub along_weight: N,
	pub expand_along: bool,
	pub shrink_along: bool,
	pub expand_across: bool,
	pub shrink_across: bool,
}

pub struct Flow<N: Num, C: Copy + 'static> {
	pub widgets: RefCell<Vec<Rc<Widget<N, C>>>>,
	pub axis: Cell<Axis>,

	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Point<N>>,
}

impl<N: Num, C: Copy + 'static> Flow<N, C> {
	pub fn new(axis: Axis, rect: Rect<N>) -> Rc<Self> {
		Rc::new(Self {
			widgets: RefCell::new(Vec::new()),
			rect: Cell::new(rect),
			axis: Cell::new(axis),
			measured: Cell::new(Point::new(N::zero(), N::zero())),
		})
	}
	pub fn vertical() -> Rc<Self> {
		Self::new(Axis::Vertical, Rect::new())
	}
	pub fn horizontal() -> Rc<Self> {
		Self::new(Axis::Horizontal, Rect::new())
	}
}

impl<N: Num, C: Copy + 'static> Container for Flow<N, C> {
	type Storage = Vec<Rc<Widget<N, C>>>;
	type Item = Rc<Widget<N, C>>;

	fn children(&self) -> Ref<Self::Storage> {
		self.widgets.borrow()
	}
	fn children_mut(&self) -> RefMut<Self::Storage> {
		self.widgets.borrow_mut()
	}
	fn add(&self, w: Self::Item) {
		let mut widgets = self.children_mut();
		widgets.push(w);
	}
	fn insert(&self, index: usize, w: Self::Item) {
		let mut widgets = self.widgets.borrow_mut();
		widgets.insert(index, w);
	}
	fn remove(&self, index: usize) -> Self::Item {
		let mut widgets = self.widgets.borrow_mut();
		widgets.remove(index)
	}
}

impl<N, C> Widget<N, C> for Flow<N, C>
	where N: Num, C: Copy + 'static
{
	fn bounds(&self) -> &Cell<Rect<N>> { &self.rect }
	fn measured_size(&self) -> &Cell<Point<N>> { &self.measured }

	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, focused: bool) {
		self.container_paint(ctx, origin, focused)
	}
	fn event(&self, event: Event<N>, origin: Point<N>, focused: bool, redraw: &Cell<bool>) -> bool {
		self.container_event(event, origin, focused, redraw)
	}

	fn measure(&self, mut width: Option<N>, mut height: Option<N>) {
		let axis = self.axis.get();

		match axis {
			Axis::Horizontal => width = None,
			Axis::Vertical => height = None,
		}

		let mut size = Point::new(N::zero(), N::zero());

		let widgets = self.widgets.borrow();
		for w in widgets.iter() {
			w.measure(width, height);
			let ms = w.measured_size().get();
			match axis {
				Axis::Horizontal => {
					size.x += ms.x;
					size.y = size.y.max(ms.y);
				}
				Axis::Vertical => {
					size.y += ms.y;
					size.x = size.x.max(ms.x);
				}
			}
		}
		self.measured.set(size);
	}

	fn layout(&self) {
		let rect = self.rect.get();
		let axis = self.axis.get();
		let zero = N::zero();

		let mut extra = match axis {
			Axis::Horizontal => rect.dx(),
			Axis::Vertical   => rect.dy(),
		};

		let mut total_expand_weight = zero;
		let mut total_shrink_weight = zero;

		let widgets = self.widgets.borrow();

		for c in widgets.iter() {
			let mut con = false;
			downcast(c.layout_data(), |d: &FlowData<N>| {
				let aw = d.along_weight;
				if aw > zero {
					con = aw == zero;
					if d.expand_along {
						total_expand_weight += aw;
					}
					if d.shrink_along {
						total_shrink_weight += aw;
					}
				}
			});
			if con {
				continue;
			}

			let size = c.measured_size().get();
			extra -= match axis {
				Axis::Horizontal => size.x,
				Axis::Vertical   => size.y,
			};
		}

		let expand = extra > zero && total_expand_weight != zero;
		let shrink = extra < zero && total_shrink_weight != zero;

		let mut total_weight = zero;
		if expand {
			total_weight = total_expand_weight;
		}
		if shrink {
			total_weight = total_shrink_weight;
		}

		let mut p = Point::new(zero, zero);
		for c in widgets.iter() {
			let mut q = Point::from_coordinates(p.coords + c.measured_size().get().coords);
			downcast(c.layout_data(), |d: &FlowData<N>| {
				if d.along_weight > zero && (expand && d.expand_along || shrink && d.shrink_along) {
					let delta = extra * d.along_weight / total_weight;
					extra -= delta;
					total_weight -= d.along_weight;
					match axis {
						Axis::Horizontal => {
							q.x += delta;
							q.x = q.x.max(p.x);
						}
						Axis::Vertical => {
							q.y += delta;
							q.y = q.y.max(p.y);
						}
					}
				}
				match axis {
					Axis::Horizontal => q.y = stretch_across(q.y, rect.dy(), d.expand_across, d.shrink_across),
					Axis::Vertical   => q.x = stretch_across(q.x, rect.dx(), d.expand_across, d.shrink_across),
				}
			});

			c.bounds().set(Rect { min: p, max: q });
			c.layout();

			match axis {
				Axis::Horizontal => p.x = q.x,
				Axis::Vertical   => p.y = q.y,
			}
		}
	}
}

fn downcast<T: 'static, F: FnOnce(&T)>(d: Option<Ref<Any>>, f: F) {
	match d {
		Some(d) => {
			match d.downcast_ref::<T>() {
				Some(d) => f(d),
				None => (),
			}
		}
		None => {}
	}
}

fn stretch_across<N: Num>(child: N, parent: N, expand: bool, shrink: bool) -> N{
	if (expand && child < parent) || (shrink && child > parent) {
		parent
	} else {
		child
	}
}