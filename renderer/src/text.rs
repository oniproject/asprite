use math::*;
use rusttype::{Font, Scale, point, PositionedGlyph};

pub struct Text<'a> {
	glyphs: Vec<PositionedGlyph<'a>>,
	text: String,
	font: &'a Font<'a>,
	scale: Scale,
}

impl<'a> Text<'a> {
	pub fn new<S>(font: &'a Font, size: f32, text: S) -> Self
		where S: ToString
	{
		Self {
			text: text.to_string(),
			glyphs: Vec::new(),
			font,
			scale: Scale::uniform(size),
		}
	}

	pub fn lay(mut self, pos: Vector2<f32>, width: u32) -> Self {
		self.layout(pos, width);
		self
	}

	pub fn glyphs(&self) -> &[PositionedGlyph<'a>] {
		&self.glyphs
	}

	pub fn set_size(&mut self, size: f32) {
		self.glyphs.clear();
		self.scale = Scale::uniform(size);
	}

	pub fn set_text<S: ToString>(&mut self, text: S) {
		self.glyphs.clear();
		self.text = text.to_string();
	}

	pub fn layout(&mut self, pos: Vector2<f32>, width: u32) {
		use unicode_normalization::UnicodeNormalization;

		self.glyphs.clear();

		let v_metrics = self.font.v_metrics(self.scale);
		let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
		let mut caret = point(pos.x, pos.y + v_metrics.ascent);
		let mut last_glyph_id = None;

		for c in self.text.nfc() {
			if c.is_control() {
				match c {
					'\n' => caret = point(pos.x, caret.y + advance_height),
					_ => {}
				}
				continue;
			}
			let base_glyph = match self.font.glyph(c) {
				Some(glyph) => glyph,
				None => continue,
			};
			if let Some(id) = last_glyph_id.take() {
				caret.x += self.font.pair_kerning(self.scale, id, base_glyph.id());
			}
			last_glyph_id = Some(base_glyph.id());
			let mut glyph = base_glyph.scaled(self.scale).positioned(caret);
			if let Some(bb) = glyph.pixel_bounding_box() {
				if bb.max.x - pos.x as i32 > width as i32 {
					caret = point(pos.x, caret.y + advance_height);
					glyph = glyph.into_unpositioned().positioned(caret);
					last_glyph_id = None;
				}
			}
			caret.x += glyph.unpositioned().h_metrics().advance_width;
			self.glyphs.push(glyph);
		}
	}
}

pub fn meashure_text_line(font: &Font, scale: f32, text: &str) -> Vector2<f32> {
	use unicode_normalization::UnicodeNormalization;

	let scale = Scale::uniform(scale);
	let v_metrics = font.v_metrics(scale);
	let height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

	let mut last_glyph_id = None;

	let iter = text.nfc().filter(|c| !c.is_control());

	let mut width = 0.0;
	for c in iter {
		let glyph = match font.glyph(c) {
			Some(glyph) => glyph,
			None => continue,
		};
		if let Some(id) = last_glyph_id.take() {
			width += font.pair_kerning(scale, id, glyph.id());
		}
		last_glyph_id = Some(glyph.id());
		width += glyph.scaled(scale).h_metrics().advance_width;
	}

	Vector2::new(width, height)
}
