#![allow(dead_code)]
#![feature(step_trait)]

extern crate math;
//extern crate num_traits;
//extern crate nalgebra as na;

use math::*;

pub use self::flow::{Flow, FlowData};
pub use self::button::Button;
pub use self::label::Label;
pub use self::root::Root;
pub use self::graphics::{Graphics, Command, TextureManager};

use self::event::Event;

pub mod graphics;

mod event;
mod theme;
mod check_set;
mod root;
mod button;
mod label;
mod flow;

use std::any::Any;
use std::cell::{Ref, RefMut, Cell};
use std::rc::Rc;
use std::ops::Deref;

#[derive(Clone)]
pub enum Mouse<N: BaseNum> {
	Move(Point2<N>),
	Press(Point2<N>),
	Release(Point2<N>),
}

pub fn example() -> Root<f32, u32> {
	let r = Rect::with_size(800.0, 100.0, 420.0, 500.0);
	let root = Root::new(r);

	let list = Flow::vertical();
	list.xy(0.0, 30.0);
	list.wh(420.0, 500.0);

	for i in 0..5 {
		let btn = Button::new(format!("fuck #{}", i), move |_| {
			println!("fuck u #{}", i);
		});
		btn.wh(60.0, 20.0);
		list.add(btn);
	}

	for i in 0..5 {
		let text = Label::new(format!("fuck #{}", i));
		text.wh(60.0, 20.0);
		list.add(text);
	}

	root.add(list);
	root.measure();
	root.layout();
	root
}

pub trait Widget<N: BaseNum, C: Copy + 'static>: Any {
	fn bounds(&self) -> &Cell<Rect<N>>;
	fn measured_size(&self) -> &Cell<Vector2<N>>;
	fn layout(&self) {}
	fn paint(&self, ctx: &mut Graphics<N, C>, origin: Vector2<N>, focused: bool);
	fn event(&self, _event: Event<N>, _origin: Vector2<N>, focused: bool, _redraw: &Cell<bool>) -> bool {
		focused
	}

	fn layout_data(&self) -> Option<Ref<Any>> { None }

	fn measure(&self, _w: Option<N>, _h: Option<N>) {
		self.measured_size().set(Vector2::zero())
	}

	fn wh(&self, w: N, h: N) {
		let r = self.bounds().get();
		self.bounds().set(r.wh(w, h));
	} 
	fn xy(&self, x: N, y: N) {
		let r = self.bounds().get();
		self.bounds().set(r.xy(x, y));
	} 
}

pub trait Container {
	type Storage: IntoIterator<Item=Self::Item>;
	type Item;
	fn children(&self) -> Ref<Self::Storage>;
	fn children_mut(&self) -> RefMut<Self::Storage>;
	fn add(&self, w: Self::Item);
	fn insert(&self, index: usize, w: Self::Item);
	fn remove(&self, index: usize) -> Self::Item;

	fn container_event<N, C, Item>(&self, event: Event<N>, origin: Vector2<N>, mut focused: bool, redraw: &Cell<bool>) -> bool
		where
			N: BaseNum,
			C: Copy + 'static,
			Self: Widget<N, C> + Container<Item=Item, Storage=Vec<Item>>,
			Item: Deref<Target=Widget<N, C>>
	{
		let origin = self.bounds().get().min + origin;
		let origin = Vector2::new(origin.x, origin.y);
		for c in self.children().iter() {
			focused = focused || c.event(event, origin, false, redraw);
		}
		focused
	}

	fn container_paint<N, C, Item>(&self, ctx: &mut Graphics<N, C>, origin: Vector2<N>, focused: bool)
		where
			N: BaseNum,
			C: Copy + 'static,
			Self: Widget<N, C> + Container<Item=Item, Storage=Vec<Item>>,
			Item: Deref<Target=Widget<N, C>>
	{
		let origin = self.bounds().get().min + origin;
		let origin = Vector2::new(origin.x, origin.y);
		for c in self.children().iter() {
			c.paint(ctx, origin, focused && false);
		}
	}
}

pub trait Shell {
	type Item;
	fn child(&self) -> Option<Self::Item>;

	fn shell_event<N, C, Item>(&self, event: Event<N>, origin: Vector2<N>, focused: bool, redraw: &Cell<bool>) -> bool
		where
			N: BaseNum,
			C: Copy + 'static,
			Self: Widget<N, C> + Shell<Item=Item>,
			Item: Deref<Target=Widget<N, C>>
	{
		let origin = self.bounds().get().min + origin;
		let origin = Vector2::new(origin.x, origin.y);
		match self.child() {
			Some(c) => c.event(event, origin, false, redraw) || focused,
			None => focused,
		}
	}

	fn shell_paint<N, C, Item>(&self, ctx: &mut Graphics<N, C>, origin: Vector2<N>, focused: bool)
		where
			N: BaseNum,
			C: Copy + 'static,
			Self: Widget<N, C> + Shell<Item=Item>,
			Item: Deref<Target=Widget<N, C>>
	{
		let origin = self.bounds().get().min + origin;
		let origin = Vector2::new(origin.x, origin.y);
		if let Some(c) = self.child() {
			c.paint(ctx, origin, focused && false);
		}
	}
}