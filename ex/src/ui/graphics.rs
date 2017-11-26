use math::*;
use super::*;

pub trait Graphics {
	type Texture;
	type Color: Copy;

	fn texture_dimensions(&self, texture: &Self::Texture) -> Vector2<f32>;
	fn quad(&self, color: Self::Color, rect: &Rect<f32>);
	fn texture(&self, texture: &Self::Texture, rect: &Rect<f32>);
	fn texture_frame(&self, texture: &Self::Texture, rect: &Rect<f32>, frame: &Rect<f32>);
	fn measure_text(&self, text: &str) -> Vector2<f32>;
	fn text(&self, base: Point2<f32>, color: Self::Color, text: &str);

	fn hovered_widget(&self) -> Option<Id>;
	fn set_hovered_widget(&self, Id);
}
