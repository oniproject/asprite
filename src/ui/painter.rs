use math::{Rect, Point2, Vector2};

pub trait Painter<D> where D: ?Sized {
    fn paint(&self, draw: &D, rect: Rect<f32>);
}

pub trait Graphics {
    type Texture: Copy;
    type Color: Copy;

    fn quad(&self, color: Self::Color, rect: Rect<f32>);

    fn texture(&self, texture: Self::Texture, rect: Rect<f32>);
    fn texture_dimensions(&self, texture: Self::Texture) -> Vector2<f32>;
    fn texture_frame(&self, texture: Self::Texture, rect: Rect<f32>, frame: Rect<f32>);

    fn measure_text(&self, text: &str) -> Vector2<f32>;
    fn text(&self, base: Point2<f32>, color: Self::Color, text: &str);

    fn set_hovered(&self);

    fn clip(&self, r: Rect<i16>);
    fn unclip(&self);
}

#[derive(Clone, Copy)]
pub struct NoDrawer;
impl<D> Painter<D> for NoDrawer where D: ?Sized + Graphics {
    fn paint(&self, _draw: &D, _rect: Rect<f32>) {}
}

#[derive(Clone, Copy)]
pub struct ColorDrawer<D: ?Sized + Graphics>(pub D::Color);
impl<D> Painter<D> for ColorDrawer<D> where D: ?Sized + Graphics {
    fn paint(&self, draw: &D, rect: Rect<f32>) {
        draw.quad(self.0, rect);
    }
}

pub struct TextureDrawer<D: ?Sized + Graphics>(pub D::Texture);
impl<D> Painter<D> for TextureDrawer<D> where D: ?Sized + Graphics {
    fn paint(&self, draw: &D, rect: Rect<f32>) {
        draw.texture(self.0, rect);
    }
}

// all sizes in pixels
pub struct NinePatch<D: ?Sized + Graphics> {
    pub tex: D::Texture,
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl<D> Painter<D> for NinePatch<D> where D: ?Sized + Graphics {
    fn paint(&self, draw: &D, rect: Rect<f32>) {
        let top = self.top;
        let bottom = self.bottom;
        let left = self.left;
        let right = self.right;

        let m = self.tex;
        let dim = draw.texture_dimensions(m);

        #[inline(always)]
        fn hsplit(r: Rect<f32>, left: f32, right: f32) -> (f32, f32) {
            (r.min.x + left, r.max.x - right)
        }

        #[inline(always)]
        fn vsplit(r: Rect<f32>, top: f32, bottom: f32) -> (f32, f32) {
            (r.min.y + top, r.max.y - bottom)
        }

        // FIXME: ?
        let x = rect.dx() / dim.x;
        let y = rect.dy() / dim.y;

        let (l, r) = hsplit(rect, left * x, right * x);
        let (t, b) = vsplit(rect, top * y, bottom * y);

        let frame = Rect::default()
            .pad_max_x(-dim.x)
            .pad_max_y(-dim.y);

        let (_l, _r) = hsplit(frame, left, right);
        let (_t, _b) = vsplit(frame, top, bottom);

        // middle
        draw.texture_frame(
            m,
            Rect::from_coords(l, t, r, b),
            Rect::from_coords(_l, _t, _r, _b),
        );

        // left top
        draw.texture_frame(m,
            Rect::from_coords(rect.min.x, rect.min.y, l, t),
            Rect::from_coords(0.0, 0.0, _l, _t),
        );
        // right top
        draw.texture_frame(m,
            Rect::from_coords(r, rect.min.y, rect.max.x, t),
            Rect::from_coords(_r, 0.0, dim.x, _t),
        );
        // right bottom
        draw.texture_frame(m,
            Rect::from_coords(r, b, rect.max.x, rect.max.y),
            Rect::from_coords(_r, _b, dim.x, dim.y),
        );
        // left bottom
        draw.texture_frame(m,
            Rect::from_coords(rect.min.x, b, l, rect.max.y),
            Rect::from_coords(0.0, _b, _l, dim.y),
        );

        // left
        draw.texture_frame(m,
            Rect::from_coords(rect.min.x, rect.min.y, l, rect.max.y),
            Rect::from_coords(0.0, 0.0, _l, dim.y),
        );
        // right
        draw.texture_frame(m,
            Rect::from_coords(r, rect.min.y, rect.max.x, rect.max.y),
            Rect::from_coords(_r, 0.0, dim.x, dim.y),
        );
        // top
        draw.texture_frame(m,
            Rect::from_coords(rect.min.x, rect.min.y, rect.max.x, t),
            Rect::from_coords(0.0, 0.0, dim.x, _t),
        );
        // bottom
        draw.texture_frame(m,
            Rect::from_coords(rect.min.x, b, rect.max.x, rect.max.y),
            Rect::from_coords(0.0, _b, dim.x, dim.y),
        );
    }
}
