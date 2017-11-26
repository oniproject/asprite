use math::*;

use std::ops::{Generator, GeneratorState};

#[derive(Copy, Clone, Debug)]
pub enum Axis {
	Horizontal,
	Vertical,
}

#[derive(Copy, Clone, Debug)]
pub struct Flow {
	pub measured_size: Vector2<f32>,

	pub along_weight: f32,

	pub expand_along: bool,
	pub shrink_along: bool,
	pub expand_across: bool,
	pub shrink_across: bool,
}

impl Flow {
	pub fn with_wh(w: f32, h: f32) -> Self {
		Self::with_size(Vector2::new(w, h))
	}
	pub fn with_size(size: Vector2<f32>) -> Self {
		Self {
			measured_size: size,
			along_weight: 0.0,
			expand_along: false,
			shrink_along: false,
			expand_across: false,
			shrink_across: false,
		}
	}

	pub fn along_weight(mut self, w: f32) -> Self {
		self.along_weight = w;
		self
	}
	pub fn expand_along(mut self) -> Self {
		self.expand_along = true;
		self
	}
	pub fn shrink_along(mut self) -> Self {
		self.shrink_along = true;
		self
	}
	pub fn expand_across(mut self) -> Self {
		self.expand_across = true;
		self
	}
	pub fn shrink_across(mut self) -> Self {
		self.shrink_across = true;
		self
	}
}

#[inline]
fn stretch_across(child: f32, parent: f32, expand: bool, shrink: bool) -> f32 {
	if (expand && child < parent) || (shrink && child > parent) {
		parent
	} else {
		child
	}
}

#[test]
fn gen_measure() {
	let widgets = &[
		Flow::with_wh(10.0, 5.0),
		Flow::with_wh(1.0, 15.0),
	];
	println!("from: {:?}", widgets);

	{
		let axis = Axis::Horizontal;
		let size = axis.measure(widgets);
		println!("h: {:?}", size);
		assert_eq!(size, Vector2::new(11.0, 15.0));

		let mut lay = axis.layout(Rect::new().dim(size), widgets);

		println!("{:?}", lay.next());
		println!("{:?}", lay.next());
		println!("{:?}", lay.next());
	}

	{
		let axis = Axis::Vertical;
		let size = axis.measure(widgets);
		println!("v: {:?}", size);
		assert_eq!(size, Vector2::new(10.0, 20.0));

		let mut lay = axis.layout(Rect::new().dim(size), widgets);

		println!("{:?}", lay.next());
		println!("{:?}", lay.next());
		println!("{:?}", lay.next());
	}
}

impl Axis {
	pub fn measure(self, widgets: &[Flow]) -> Vector2<f32> {
		let f: fn (s: Vector2<f32>, v: Vector2<f32>) -> Vector2<f32> =
			match self {
				Axis::Horizontal => move |s, v| Vector2::new(s.x + v.x, s.y.max(v.y)),
				Axis::Vertical   => move |s, v| Vector2::new(s.x.max(v.x), s.y + v.y),
			};

		widgets.iter()
			.map(|c| c.measured_size)
			.fold(Vector2::zero(), f)
	}

	pub fn layout<'a>(self, rect: Rect<f32>, widgets: &'a [Flow]) -> impl Iterator<Item=Rect<f32>> + 'a {
		let mut extra = match self {
			Axis::Horizontal => rect.dx(),
			Axis::Vertical   => rect.dy(),
		};

		let mut total_expand_weight = 0.0;
		let mut total_shrink_weight = 0.0;

		for c in widgets.iter() {
			if c.along_weight <= 0.0 {
				continue;
			}
			if c.expand_along {
				total_expand_weight += c.along_weight;
			}
			if c.shrink_along {
				total_shrink_weight += c.along_weight;
			}

			let size = c.measured_size;
			extra -= match self {
				Axis::Horizontal => size.x,
				Axis::Vertical   => size.y,
			};
		}

		let expand = extra > 0.0 && total_expand_weight != 0.0;
		let shrink = extra < 0.0 && total_shrink_weight != 0.0;

		let mut total_weight = 0.0;
		if expand {
			total_weight = total_expand_weight;
		}
		if shrink {
			total_weight = total_shrink_weight;
		}

		let gen = move || {
			let mut p = Vector2::zero();
			let mut q = Vector2::zero();
			for c in widgets.iter() {
				match self {
					Axis::Horizontal => p.x = q.x,
					Axis::Vertical   => p.y = q.y,
				}

				q = p + c.measured_size;

				if c.along_weight > 0.0 && (expand && c.expand_along || shrink && c.shrink_along) {
					let delta = extra * c.along_weight / total_weight;
					extra -= delta;
					total_weight -= c.along_weight;
					match self {
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

				match self {
					Axis::Horizontal => q.y = stretch_across(q.y, rect.dy(), c.expand_across, c.shrink_across),
					Axis::Vertical   => q.x = stretch_across(q.x, rect.dx(), c.expand_across, c.shrink_across),
				}

				yield Rect::with_coords(p.x, p.y, q.x, q.y);
			}
		};

		LayoutIter { gen }
	}
}

pub struct LayoutIter<G>
	where G: Generator<Yield=Rect<f32>, Return=()>
{
	gen: G,
}

impl<G> Iterator for LayoutIter<G>
	where G: Generator<Yield=Rect<f32>, Return=()>
{
	type Item = Rect<f32>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.gen.resume() {
			GeneratorState::Yielded(rect) => Some(rect),
			GeneratorState::Complete(()) => None,
		}
	}
}
