use super::*;
use super::layout::*;
use math::*;

use std::rc::Rc;

pub struct ContextBuilder<'a, 'b, D: ?Sized + Graphics + 'a>
    where 'a: 'b
{
    root: &'b Context<'a, D>,
    rect: Rect<f32>,
    range: Option<usize>,
}

impl<'a, 'b, D: ?Sized + Graphics + 'a> ContextBuilder<'a, 'b, D> {
    fn new(root: &'b Context<'a, D>) -> Self {
        Self {
            root,
            rect: root.rect,
            range: None,
        }
    }

    pub fn with_rect(mut self, rect: Rect<f32>) -> Self {
        self.rect = rect;
        self
    }

    pub fn align(mut self, align: Vector2<f32>, size: Vector2<f32>) -> Self {
        let rect = self.root.rect;
        let p = rect_align(rect, align, size);
        self.rect = Rect::from_min_dim(p, size);
        self
    }

    pub fn transform(mut self, anchor: Rect<f32>, offset: Rect<f32>) -> Self {
        let rect = self.root.rect;
        self.rect = rect_transform(rect, anchor, offset);
        self
    }

    pub fn with_range(mut self, count: usize) -> Self {
        self.range = Some(count);
        self
    }

    pub fn build(self) -> Context<'a, D> {
        let Self { root, rect, range } = self;
        let generator = range
            .and_then(|range| root.generator.range(range))
            .map(|gen| Rc::new(gen))
            .unwrap_or_else(|| root.generator.clone());
        Context { generator, rect, .. *root }
    }
}

pub struct Context<'a, D: ?Sized + Graphics + 'a> {
    draw: &'a D,
    generator: Rc<Generator>,
    rect: Rect<f32>,
    mouse: Mouse,
}

impl<'a, D: ?Sized + Graphics + 'a> Context<'a, D> {
    pub fn new(draw: &'a D, rect: Rect<f32>, mouse: Mouse) -> Self {
        Self {
            rect,
            draw,
            mouse,
            generator: Rc::new(Generator::new()),
        }
    }

    pub fn in_range(&self, id: Id) -> bool {
        self.generator.in_range(id)
    }

    pub fn sub<'b>(&'b self) -> ContextBuilder<'a, 'b, D> {
        ContextBuilder::new(self)
    }

    pub fn sub_range(&self, range: usize) -> Self {
        self.sub().with_range(range).build()
    }

    pub fn align(&self, align: Vector2<f32>, size: Vector2<f32>) -> Self {
        self.sub().align(align, size).build()
    }

    pub fn transform(&self, anchor: Rect<f32>, offset: Rect<f32>) -> Self {
        self.sub().transform(anchor, offset).build()
    }

    pub fn sub_rect(&self, rect: Rect<f32>) -> Self {
        Self {
            rect,
            generator: self.generator.clone(),
            .. *self
        }
    }

    pub fn split_x(&self, x: f32) -> (Self, Self) {
        let (a, b) = self.rect.split_x(x);
        let a = self.sub_rect(a);
        let b = self.sub_rect(b);
        (a, b)
    }

    pub fn split_y(&self, y: f32) -> (Self, Self) {
        let (a, b) = self.rect.split_y(y);
        let a = self.sub_rect(a);
        let b = self.sub_rect(b);
        (a, b)
    }

    pub fn horizontal_flow(&self, x: f32, y: f32, widgets: &'a [Flow]) -> impl Iterator<Item=Context<'a, D>> {
        self.layout(Axis::Horizontal, x, y, widgets)
    }
    pub fn vertical_flow(&self, x: f32, y: f32, widgets: &'a [Flow]) -> impl Iterator<Item=Context<'a, D>> {
        self.layout(Axis::Vertical, x, y, widgets)
    }

    pub fn label(&self, x: f32, y: f32, color: D::Color, text: &str) {
        self.label_rect(self.rect(), Vector2::new(x, y), color, text);
    }

    pub fn label_rect(&self, rect: Rect<f32>, align: Vector2<f32>, color: D::Color, text: &str) {
        let size = self.draw.measure_text(&text);
        let p = rect_align(rect, align, size);
        self.draw.text(p, color, &text);
    }

    fn layout(&self, axis: Axis, x: f32, y: f32, widgets: &'a [Flow]) -> impl Iterator<Item=Context<'a, D>> + 'a {
        let size = measure(axis, widgets);
        //let offset = ctx.rect().min.to_vec();
        let offset = rect_align(self.rect(), Vector2::new(x, y), size);
        let offset = Vector2::new(offset.x, offset.y);
        let draw = self.draw;
        let generator = self.generator.clone();
        let size = Vector2::new(self.rect().dx(), self.rect().dy());

        let mouse = self.mouse;
        layout(axis, size, widgets)
            .map(move |rect| rect.shift_xy(offset))
            .map(move |rect| Self {
                rect,
                draw,
                generator: generator.clone(),
                mouse,
            })
    }

    pub fn draw(&self) -> &'a D {
        self.draw
    }

    pub fn reserve_widget_id(&self) -> Id {
        self.generator.next().unwrap()
    }

    pub fn is_cursor_hovering(&self) -> bool {
        self.is_cursor_in_rect(self.rect)
    }
}

impl<'a, D: ?Sized + Graphics + 'a> Events for Context<'a, D> {
    fn rect(&self) -> Rect<f32> {
        self.rect
    }

    fn cursor(&self) -> Point2<f32> { self.mouse.cursor }
    fn was_pressed(&self) -> bool { self.mouse.pressed }
    fn was_released(&self) -> bool { self.mouse.released }
}

impl<'a, D: ?Sized + Graphics + 'a> Graphics for Context<'a, D> {
    type Texture = D::Texture;
    type Color = D::Color;

    fn texture_dimensions(&self, texture: Self::Texture) -> Vector2<f32> {
        self.draw.texture_dimensions(texture)
    }
    fn quad(&self, color: Self::Color, rect: Rect<f32>) {
        self.draw.quad(color, rect)
    }
    fn texture(&self, texture: Self::Texture, rect: Rect<f32>) {
        self.draw.texture(texture, rect)
    }
    fn texture_frame(&self, texture: Self::Texture, rect: Rect<f32>, frame: Rect<f32>) {
        self.draw.texture_frame(texture, rect, frame)
    }
    fn measure_text(&self, text: &str) -> Vector2<f32> {
        self.draw.measure_text(text)
    }
    fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
        self.draw.text(base, color, text)
    }
    fn set_hovered(&self) {
        self.draw.set_hovered()
    }

    fn clip(&self, r: Rect<i16>) {
        self.draw.clip(r)
    }
    fn unclip(&self) {
        self.draw.unclip()
    }
}
