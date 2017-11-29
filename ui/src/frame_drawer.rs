use super::*;
use math::*;

pub trait FrameDrawer<D: ?Sized> {
	fn draw_frame(&self, &D, Rect<f32>);
}

pub struct NoDrawer;
impl<D: ?Sized + Graphics> FrameDrawer<D> for NoDrawer {
	fn draw_frame(&self, _draw: &D, _rect: Rect<f32>) {}
}

pub struct ColorDrawer<D: ?Sized + Graphics>(D::Color);
impl<D: ?Sized + Graphics> ColorDrawer<D> {
	pub fn new(color: D::Color) -> Self {
		ColorDrawer(color)
	}
}
impl<D: ?Sized + Graphics> FrameDrawer<D> for ColorDrawer<D> {
	fn draw_frame(&self, draw: &D, rect: Rect<f32>) {
		draw.quad(self.0, &rect);
	}
}

pub struct TextureDrawer<D: ?Sized + Graphics>(D::Texture);
impl<D: ?Sized + Graphics> TextureDrawer<D> {
	pub fn new(texture: D::Texture) -> Self {
		TextureDrawer(texture)
	}
}
impl<D: ?Sized + Graphics> FrameDrawer<D> for TextureDrawer<D> {
	fn draw_frame(&self, draw: &D, rect: Rect<f32>) {
		draw.texture(&self.0, &rect);
	}
}
