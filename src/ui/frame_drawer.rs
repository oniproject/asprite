use super::*;
use math::*;

// TODO: maybe Painter like this?
//
// pub trait Painter<D: ?Sized> {
//    fn paint(&self, &D, Rect<f32>);
//}

pub trait FrameDrawer<D: ?Sized> {
    fn draw_frame(&self, &D, Rect<f32>);
}

#[derive(Clone, Copy)]
pub struct NoDrawer;
impl<D: ?Sized + Graphics> FrameDrawer<D> for NoDrawer {
    fn draw_frame(&self, _draw: &D, _rect: Rect<f32>) {}
}

pub struct ColorDrawer<D: ?Sized + Graphics>(D::Color);
impl<D: ?Sized + Graphics> ColorDrawer<D> {
    pub const fn new(color: D::Color) -> Self {
        ColorDrawer(color)
    }
}
impl<D: ?Sized + Graphics> FrameDrawer<D> for ColorDrawer<D> {
    fn draw_frame(&self, draw: &D, rect: Rect<f32>) {
        draw.quad(self.0, &rect);
    }
}

impl<D: ?Sized + Graphics> Copy for ColorDrawer<D> {}

impl<D: ?Sized + Graphics> Clone for ColorDrawer<D> {
    fn clone(&self) -> Self { ColorDrawer(self.0) }
}

pub struct TextureDrawer<D: ?Sized + Graphics>(D::Texture);
impl<D: ?Sized + Graphics> TextureDrawer<D> {
    pub const fn new(texture: D::Texture) -> Self {
        TextureDrawer(texture)
    }
}
impl<D: ?Sized + Graphics> FrameDrawer<D> for TextureDrawer<D> {
    fn draw_frame(&self, draw: &D, rect: Rect<f32>) {
        draw.texture(&self.0, &rect);
    }
}
