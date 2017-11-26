use super::*;
use math::*;

use std::cell::Cell;
use std::rc::Rc;

pub struct ContextBuilder<'a, 'b, D: ?Sized + Graphics + 'a>
	where 'a: 'b
{
	root: &'b Context<'a, D>,
	rect: Option<Rect<f32>>,
	range: Option<usize>,
}

impl<'a, 'b, D: ?Sized + Graphics + 'a> ContextBuilder<'a, 'b, D> {
	fn new(root: &'b Context<'a, D>) -> Self {
		Self {
			root,
			rect: None,
			range: None,
		}
	}

	pub fn with_rect(mut self, rect: Rect<f32>) -> Self {
		self.rect = Some(rect);
		self
	}

	pub fn transform(mut self, anchor: Rect<f32>, offset: Rect<f32>) -> Self {
		let rect = self.root.rect.get();
		self.rect = Some(rect.transform(anchor, offset));
		self
	}
	pub fn stretch(mut self) -> Self {
		self.rect = Some(self.root.rect.get());
		self
	}

	pub fn with_range(mut self, count: usize) -> Self {
		self.range = Some(count);
		self
	}

	pub fn build(self) -> Context<'a, D> {
		let Self { root, rect, range } = self;

		let rect = Cell::new(rect.unwrap_or(root.rect.get()));
		let generator = range
			.and_then(|range| root.generator.range(range))
			.map(|gen| Rc::new(gen))
			.unwrap_or_else(|| root.generator.clone());

		Context {
			rect,
			generator,
			mouse: root.mouse,
			draw: root.draw,
		}
	}
}


pub struct Context<'a, D: ?Sized + Graphics + 'a> {
	draw: &'a D,
	generator: Rc<Generator>,
	rect: Cell<Rect<f32>>,
	mouse: Mouse,
}

impl<'a, D: ?Sized + Graphics + 'a> Context<'a, D> {
	#[inline]
	pub fn new(draw: &'a D, rect: Rect<f32>, mouse: Mouse) -> Self {
		Self {
			rect: Cell::new(rect),
			generator: Rc::new(Generator::new()),
			mouse,
			draw,
		}
	}

	#[inline]
	pub fn sub<'b>(&'b self) -> ContextBuilder<'a, 'b, D> {
		ContextBuilder::new(self)
	}

	#[inline]
	pub fn draw(&self) -> &'a D {
		self.draw
	}

	#[inline]
	pub fn mouse(&self) -> & Mouse {
		&self.mouse
	}

	#[inline]
	pub fn rect(&self) -> Rect<f32> {
		self.rect.get()
	}

	#[inline]
	pub fn reserve_widget_id(&self) -> Id {
		self.generator.next().unwrap()
	}

	#[inline]
	pub fn is_cursor_hovering(&self) -> bool {
		self.mouse.check_cursor(&self.rect.get())
	}

	#[inline]
	pub fn static_cursor(&self, id: Id) {
		if self.hovered_widget().is_none() {
			if self.is_cursor_hovering() {
				self.set_hovered_widget(id);
			}
		}
	}
}

impl<'a, D: ?Sized + Graphics + 'a> Graphics for Context<'a, D> {
	type Texture = D::Texture;
	type Color = D::Color;

	#[inline]
	fn texture_dimensions(&self, texture: &Self::Texture) -> Vector2<f32> {
		self.draw.texture_dimensions(texture)
	}
	#[inline]
	fn quad(&self, color: Self::Color, rect: &Rect<f32>) {
		self.draw.quad(color, rect)
	}
	#[inline]
	fn texture(&self, texture: &Self::Texture, rect: &Rect<f32>) {
		self.draw.texture(texture, rect)
	}
	#[inline]
	fn texture_frame(&self, texture: &Self::Texture, rect: &Rect<f32>, frame: &Rect<f32>) {
		self.draw.texture_frame(texture, rect, frame)
	}
	#[inline]
	fn measure_text(&self, text: &str) -> Vector2<f32> {
		self.draw.measure_text(text)
	}
	#[inline]
	fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
		self.draw.text(base, color, text)
	}

	#[inline]
	fn hovered_widget(&self) -> Option<Id> {
		self.draw.hovered_widget()
	}
	#[inline]
	fn set_hovered_widget(&self, id: Id) {
		self.draw.set_hovered_widget(id)
	}
}

impl<'a, D: ?Sized + Graphics + 'a> MouseEvent for Context<'a, D> {
	#[inline]
	fn was_pressed(&self) -> bool {
		self.mouse.was_pressed()
	}
	#[inline]
	fn was_released(&self) -> bool {
		self.mouse.was_released()
	}
}
