use common::*;
use super::*;
use std::cell::Cell;
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

pub struct Flow<N: SignedInt, C: Copy + 'static> {
	pub widgets: Vec<Rc<Widget<N, C>>>,
	pub rect: Cell<Rect<N>>,
	pub measured: Cell<Point<N>>,
	pub axis: Cell<Axis>,
}

impl<N: SignedInt, C: Copy + 'static> Measured<N> for Flow<N, C> {
	fn measured_size(&self) -> &Cell<Point<N>> {
		&self.measured
	}
	fn measure(&self, mut width: Option<N>, mut height: Option<N>) {
		let axis = self.axis.get();

		match axis {
			Axis::Horizontal => width = None,
			Axis::Vertical => height = None,
		}

		let mut size = Point::new(N::zero(), N::zero());

		for w in &self.widgets {
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
}

impl<N: SignedInt, C: Copy + 'static> Flow<N, C> {
	pub fn new(axis: Axis, rect: Rect<N>) -> Self {
		Self {
			widgets: Vec::new(),
			rect: Cell::new(rect),
			axis: Cell::new(axis),
			measured: Cell::new(Point::new(N::zero(), N::zero())),
		}
	}
	pub fn vertical(rect: Rect<N>) -> Self {
		Self::new(Axis::Vertical, rect)
	}
	pub fn horizontal(rect: Rect<N>) -> Self {
		Self::new(Axis::Horizontal, rect)
	}
	pub fn add(&mut self, w: Rc<Widget<N, C>>) {
		self.widgets.push(w);
	}
}

impl<N: SignedInt, C: Copy + 'static> Layout for Flow<N, C> {
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

		for c in &self.widgets {
			let d = c.layout_data().and_then(|p| p.downcast_ref::<FlowData<N>>());
			if let Some(c) = d {
				let aw = c.along_weight;
				if aw > zero {
					if aw == zero {
						continue;
					}
					if c.expand_along {
						total_expand_weight += aw;
					}
					if c.shrink_along {
						total_shrink_weight += aw;
					}
				}
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
		for c in &self.widgets {
			let mut q = Point::from_coordinates(p.coords + c.measured_size().get().coords);
			let d = c.layout_data().and_then(|p| p.downcast_ref::<FlowData<N>>());
			if let Some(d) = d {
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
			}

			c.bounds().set(Rect { min: p, max: q });
			c.layout();

			match axis {
				Axis::Horizontal => p.x = q.x,
				Axis::Vertical   => p.y = q.y,
			}
		}
	}
}


fn stretch_across<N: Num>(child: N, parent: N, expand: bool, shrink: bool) -> N{
	if (expand && child < parent) || (shrink && child > parent) {
		parent
	} else {
		child
	}
}

impl<N: SignedInt, C: Copy + 'static> Bounds<N> for Flow<N, C> {
	fn bounds(&self) -> &Cell<Rect<N>> {
		&self.rect
	}
}

impl<N, C> Widget<N, C> for Flow<N, C>
	where N: SignedInt, C: Copy + 'static
{
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Point<N>, focused: bool) {
		let origin = Point::from_coordinates(self.rect.get().min.coords + origin.coords);
		for w in self.widgets.iter() {
			w.paint(ctx, origin, focused && false);
		}
	}
	fn event(&self, event: Event<N>, origin: Point<N>, focused: bool, redraw: &Cell<bool>) -> bool {
		let origin = Point::from_coordinates(self.rect.get().min.coords + origin.coords);
		for w in self.widgets.iter() {
			w.event(event, origin, false, redraw);
		}
		focused
	}
}