use na::{Vector2, UnitComplex};
use std::rc::Rc;

pub struct TileCell {}
pub struct Image {}

#[derive(Clone)]
pub struct Font {
	pub id: u32,

	pub bold: bool,
	pub italic: bool,
	pub underline: bool,
	pub strikeout: bool,
	pub kerning: bool,
}

#[derive(Clone)]
pub struct TextMeta {
	pub font: Font,
	pub color: u32,
	pub word_wrap: bool,
	pub align: (f32, f32),
}

#[derive(Clone)]
pub enum Kind {
	Rectangle,
	Ellipse,
	Polygon(Vec<Vector2<f32>>),
	Polyline(Vec<Vector2<f32>>),
	Cell(Rc<TileCell>),
	Text(String, TextMeta),
	Image(Rc<Image>),
}

bitflags! {
    pub struct Properties: u32 {
		const NAME             = 1 << 0;
		const KIND             = 1 << 1;
		const VISIBLE          = 1 << 2;
		const ROTATION         = 1 << 3;
		const SIZE             = 1 << 4;

		const SHAPE            = 1 << 5;

		const TEXT             = 1 << 6;
		const TEXT_FONT        = 1 << 7;
		const TEXT_ALIGNMENT   = 1 << 8;
		const TEXT_WORD_WRAP   = 1 << 9;
		const TEXT_COLOR       = 1 << 10;
		const CELL             = 1 << 11;
	}
}

pub struct Shape {
	pub id: u32,
	pub kind: Kind,
	pub name: String,
	pub visible: bool,

	pub min: Vector2<f32>,
	pub max: Vector2<f32>,
	pub rotation: UnitComplex<f32>,

	pub template: Option<Rc<Shape>>,
	pub overrided: Properties,
}

impl Shape {
	pub fn sync(&mut self) {
		if let Some(ref t) = self.template {
			if self.overrided.contains(Properties::NAME) {
				self.name = t.name.clone();
			}
			if self.overrided.contains(Properties::KIND) {
				self.kind = t.kind.clone();
			}
			if self.overrided.contains(Properties::VISIBLE) {
				self.visible = t.visible;
			}
			if self.overrided.contains(Properties::ROTATION) {
				self.rotation = t.rotation;
			}
			if self.overrided.contains(Properties::SIZE) {
				self.max = t.min - t.max;
			}

			let failed =
				Properties::TEXT |
				Properties::TEXT_FONT |
				Properties::TEXT_ALIGNMENT |
				Properties::TEXT_WORD_WRAP |
				Properties::TEXT_COLOR |
				Properties::CELL |
				Properties::SHAPE;

			if self.overrided.contains(failed) {
				unimplemented!()
			}
		}
	}
}