use std::mem::{replace, swap};
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};

use redo::{Record, Command};

use math::{Rect, Point2, Vector2};
use draw::{
    Bounded,
    CanvasRead,
    CanvasWrite,
    Frame,
    Palette,
};

use super::{
    Brush,
    BrushOwned,
    Receiver,
    brush::Shape,
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
    pub brush_rect: Rect<i32>,

    canvas: Frame,
    start: usize,
    stride: usize,
}

impl Editor {
    pub fn new(pos: Point2<i32>, image: Receiver) -> Self {
        let (w, h) = (image.width, image.height);
        let canvas = image.page(image.layer, image.frame).clone();

        Self {
            canvas,
            stride: image.width,
            start: 0,
            image: Record::new(image),
            brush: Shape::Round.gen(5, 5),
            brush_rect: Rect::from_coords_and_size(-2, -2, 5, 5),
        }
    }

    pub fn recreate(&mut self, image: Receiver) {
        let (w, h) = (image.width, image.height);
        self.canvas = image.page(image.layer, image.frame).clone();
        self.image = Record::new(image);
        self.stride = w;
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

    pub fn pal_color(&self) -> u32 {
        let m = self.image.as_receiver();
        m.palette[m.color]
    }

    pub fn transparent(&self) -> Option<u8> {
        self.image.as_receiver().current().transparent
    }

    pub fn pal(&self, color: u8) -> u32 {
        self.image.as_receiver().palette[color]
    }

    pub fn color_index(&self) -> u8 {
        self.image.as_receiver().color
    }

    pub fn take_redraw(&mut self) -> Option<Rect<i32>> {
        self.image.as_mut_receiver().take_update()
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
    unsafe fn set_pixel_unchecked(&mut self, x: i32, y: i32, color: u8) {
        let x = x as usize;
        let y = y as usize;
        let idx = self.start + x + y * self.stride;
        *self.canvas.page.get_unchecked_mut(idx) = color;
    }
}

impl CanvasRead<u8, i32> for Editor {
    #[inline(always)]
    unsafe fn get_pixel_unchecked(&self, x: i32, y: i32) -> u8 {
        let x = x as usize;
        let y = y as usize;
        let idx = self.start + x + y * self.stride;
        *self.canvas.page.get_unchecked(idx)
    }
}

impl super::Context<i32, u8> for Editor {
    fn update(&mut self, r: Rect<i32>) {
        self.image.as_mut_receiver().update(r)
    }

    fn sync(&mut self) {
        {
            let mut m = self.image.as_receiver();
            self.canvas.copy_from(&m.page(m.layer, m.frame));
        }
        self.image.as_mut_receiver().update_all();
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

    fn color(&self) -> u8 {
        self.image.as_receiver().color
    }

    fn change_color(&mut self, color: u8) {
        self.image.as_mut_receiver().color = color;
    }

    fn brush(&self) -> (Brush, Rect<i32>) {
        (&self.brush, self.brush_rect)
    }
}
