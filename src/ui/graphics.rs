use math::*;

pub trait Graphics {
    type Texture;
    type Color: Copy;

    fn quad(&self, color: Self::Color, rect: &Rect<f32>);

    fn texture(&self, texture: &Self::Texture, rect: &Rect<f32>);
    fn texture_dimensions(&self, texture: &Self::Texture) -> Vector2<f32>;
    fn texture_frame(&self, texture: &Self::Texture, rect: &Rect<f32>, frame: &Rect<f32>);

    fn measure_text(&self, text: &str) -> Vector2<f32>;
    fn text(&self, base: Point2<f32>, color: Self::Color, text: &str);

    fn set_hovered(&self);

    fn clip(&self, r: Rect<i16>);
    fn unclip(&self);
}
