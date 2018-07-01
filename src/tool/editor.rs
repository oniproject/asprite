use std::mem::swap;

use redo::{Record, Command};

use math::{Rect, Point2, Vector2};
use draw::{
    self,
    Bounded,
    CanvasRead,
    CanvasWrite,
    Frame,
    Palette,
    Shape,
};

use super::{
    Brush,
    Receiver,
};

#[derive(Debug)]
pub struct DrawCommand {
    page: Frame,
    frame: usize,
    layer: usize,
}

impl DrawCommand {
    pub fn new(frame: usize, layer: usize, page: Frame) -> Self {
        Self { frame, layer, page }
    }
    fn run(&mut self, image: &mut Receiver) -> Result<(), ()> {
        swap(&mut self.page, image.page_mut(self.frame, self.layer));
        Ok(())
    }
}

impl Command<Receiver> for DrawCommand {
    type Error = ();
    fn apply(&mut self, image: &mut Receiver) -> Result<(), Self::Error> { self.run(image) }
    fn undo(&mut self, image: &mut Receiver) -> Result<(), Self::Error> { self.run(image) }
}

pub struct Editor {
    pub image: Record<Receiver, DrawCommand>,
    pub brush: Vec<bool>,
    pub brush_shape: Shape,
    pub brush_offset: Point2<i32>,
    pub brush_size: Vector2<i32>,
    pub brush_size_old: Vector2<i32>,
    pub color: u8,

    canvas: Frame,
}

impl Editor {
    pub fn new(image: Receiver) -> Self {
        let brush_size = Vector2::new(11, 11);
        Self {
            canvas: image.page(image.layer, image.frame).clone(),
            image: Record::new(image),
            brush: draw::shape::round(brush_size.x, brush_size.y).collect(),
            brush_shape: Shape::Round,
            brush_size,
            brush_size_old: brush_size,
            brush_offset: Point2::new(-5, -5),
            color: 1,
        }
    }

    pub fn resize_brush(&mut self) {
        use draw::{View, ViewMut};
        use draw::shape::*;
        use self::Shape::*;
        use math::SliceExt;
        use std::mem::replace;

        let s = self.brush_size;
        let (w, h) = (s.x, s.y);
        let old = replace(&mut self.brush_size_old, self.brush_size);
        self.brush = match self.brush_shape {
            Round           => round(w, h).collect(),
            Square          => square(w, h).collect(),
            SieveRound      => sieve_round(w, h).collect(),
            SieveSquare     => sieve_square(w, h).collect(),
            Plus            => plus(w, h).collect(),
            Slash           => slash(w, h).collect(),
            Antislash       => antislash(w, h).collect(),
            HorizontalBar   => horizontal_bar(w, h).collect(),
            VerticalBar     => vertical_bar(w, h).collect(),
            Cross           => cross(w, h).collect(),
            Diamond         => diamond(w, h).collect(),
            Custom => unsafe {
                let src: View<bool, _> = View::new(&self.brush.cast(), old.x, old.y);
                let mut data = vec![false; (w * h) as usize];
                {
                    let d = &mut data.cast_mut();
                    let mut dst = ViewMut::new(d, w, h);
                    let r = dst.bounds().intersect(src.bounds()).unwrap();
                    println!("w: {} h: {} r: {:?}", w, h, r);
                    draw::view::copy(&mut dst, r, &src, Point2::new(0, 0));
                }
                data
            }
        };
    }

    pub fn recreate(&mut self, image: Receiver) {
        self.canvas = image.current().clone();
        self.image = Record::new(image);
    }

    pub fn zoom(&self) -> i32 {
        self.image.as_receiver().zoom
    }

    pub fn pos(&self) -> Point2<i32> {
        self.image.as_receiver().pos
    }

    pub fn size(&self) -> Vector2<i32> {
        let m = self.image.as_receiver();
        Vector2::new(m.width as i32, m.height as i32)
    }

    pub fn rect(&self) -> Rect<i32> {
        self.image.as_receiver().rect()
    }

    pub fn take_created(&mut self) -> bool {
        let m = self.image.as_mut_receiver();
        let c = m.created;
        m.created = true;
        c
    }

    pub fn transparent(&self) -> Option<u8> {
        self.image.as_receiver().current().transparent
    }

    pub fn pal(&self, color: u8) -> u32 {
        self.image.as_receiver().palette[color]
    }

    pub fn redo(&mut self) {
        use super::Context;
        self.image.redo();
        self.sync();
    }

    pub fn undo(&mut self) {
        use super::Context;
        self.image.undo();
        self.sync();
    }

    pub fn draw_pages<F: FnMut(&Frame, &Palette<u32>)>(&self, mut f: F) {
        let image = self.image.as_receiver();
        let current_layer = image.layer;
        let current_frame = image.frame;
        for (layer_id, layer) in image.data.iter().enumerate() {
            if !layer.visible {
                continue;
            }
            for (frame_id, _) in layer.frames.iter().enumerate() {
                let is_canvas = layer_id == current_layer && frame_id == current_frame;
                let page = if is_canvas {
                    Some(&self.canvas)
                } else {
                    Some(image.page(layer_id, frame_id))
                };
                if let Some(page) = page {
                    f(&page, &image.palette)
                }
            }
        }
    }
}

impl Bounded<i32> for Editor {
    #[inline(always)]
    fn bounds(&self) -> Rect<i32> {
        let min = Point2::new(0, 0);
        let dim = self.size();
        Rect::from_min_dim(min, dim)
    }
}

impl CanvasWrite<u8, i32> for Editor {
    #[inline(always)]
    unsafe fn set_unchecked(&mut self, x: i32, y: i32, color: u8) {
        self.canvas.view_mut().set_unchecked(x, y, color)
    }
}

impl CanvasRead<u8, i32> for Editor {
    #[inline(always)]
    unsafe fn at_unchecked(&self, x: i32, y: i32) -> u8 {
        self.canvas.view().at_unchecked(x, y)
    }
}

impl super::Context<i32, u8> for Editor {
    fn sync(&mut self) {
        let m = self.image.as_receiver();
        self.canvas.copy_from(m.current());
    }

    fn start(&mut self) {
        self.sync();
    }

    fn commit(&mut self) {
        let page = self.canvas.clone();
        let layer = self.image.as_receiver().layer;
        let frame = self.image.as_receiver().frame;
        let _ = self.image
            .apply(DrawCommand::new(layer, frame, page.clone())).unwrap();
        self.sync();
    }

    fn rollback(&mut self) {
        self.sync();
    }

    fn change_color(&mut self, color: u8) {
        self.color = color;
    }
}

impl super::PreviewContext<i32, u8> for Editor {
    fn brush(&self) -> (Brush, Rect<i32>) {
        (&self.brush, Rect::from_min_dim(self.brush_offset, self.brush_size))
    }
    fn color(&self) -> u8 {
        self.color
    }
}
