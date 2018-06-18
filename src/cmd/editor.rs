use math::{Rect, Point2, Vector2};
use tool::Context;
use tool::Brush;

use super::DrawCommand;

use draw::{
    Bounded,
    CanvasRead,
    CanvasWrite,
    Sprite,
    Frame,
    Palette,
};

use redo::Record;
use std::mem::replace;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};

pub type ImageCell = Arc<Mutex<Record<Sprite, DrawCommand>>>;
pub fn image_cell(sprite: Sprite) -> ImageCell {
    Arc::new(Mutex::new(Record::new(sprite)))
}

pub struct Editor {
    pub image: ImageCell,
    redraw: Option<Rect<i32>>,

    pub brush_rect: Rect<i16>,
    pub brush_offset: Vector2<i16>,

    canvas: Frame,
    rect: Rect<i32>,
    start: usize,
    stride: usize,
}

impl Editor {
    pub fn new(image: ImageCell) -> Self {
        let (w, h) = {
            let m = image.lock().unwrap();
            let m = m.as_receiver();
            (m.width, m.height)
        };
        let rect = Rect::from_coords_and_size(0, 0, w as i32, h as i32);
        Self {
            image,
            rect,

            canvas: Frame::new(w, h),
            start: 0,
            stride: w,

            redraw: Some(rect),
            brush_rect: Rect::from_coords_and_size(0, 0, 3, 3),
            brush_offset: Vector2::new(-1, -1),
        }
    }

    pub fn take_redraw(&mut self) -> Option<Rect<i32>> {
        self.redraw.take()
    }

    pub fn sprite(&self) -> MutexGuard<Record<Sprite, DrawCommand>> {
        self.image.lock().unwrap()
    }

    pub fn size(&self) -> Vector2<i32> {
        Vector2::new(self.rect.dx() as i32, self.rect.dy() as i32)
    }

    pub fn redo(&mut self) {
        self.image.lock().unwrap().redo();
        self.sync();
    }

    pub fn undo(&mut self) {
        self.image.lock().unwrap().undo();
        self.sync();
    }

    pub fn draw_pages<F: FnMut(&Frame, &Palette<u32>)>(&self, mut f: F) {
        let image = self.sprite();
        let image = image.as_receiver();
        let current_layer = image.layer.get();
        let current_frame = image.frame.get();
        for (layer_id, layer) in image.data.iter().enumerate() {
            if !layer.visible.get() {
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
        self.rect
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

impl Context<i32, u8> for Editor {
    fn update(&mut self, r: Rect<i32>) {
        self.redraw = match self.redraw {
            Some(r) => r.union(r),
            None => Some(r),
        };
    }

    fn sync(&mut self) {
        let m = self.image.lock().unwrap();
        let m = m.as_receiver();
        let (layer, frame) = {
            (m.layer.get(), m.frame.get())
        };
        self.canvas.copy_from(&m.page(layer, frame));
        self.redraw = Some(self.rect);
    }

    fn start(&mut self) -> u8 {
        self.sync();
        self.sprite().as_receiver().color.get()
    }
    fn commit(&mut self) {
        let page = self.canvas.clone();
        let (layer, frame) = {
            let m = self.sprite();
            let m = m.as_receiver();
            (m.layer.get(), m.frame.get())
        };
        let _ = self.image.lock().unwrap()
            .apply(DrawCommand::new(layer, frame, page.clone())).unwrap();
        self.sync();
    }
    fn rollback(&mut self) {
        self.sync();
    }

    fn change_color(&mut self, color: u8) {
        self.sprite().as_receiver().color.set(color);
    }

    fn brush(&self) -> (Brush<u8>, Rect<i32>) {
        static BRUSH: &[bool] = &[
            true,  false,  true,
            false,  true, false,
            true,  false,  true,
        ];
        let r = Rect::from_coords_and_size(-1, -1, 3, 3);
        (Brush::Mask(BRUSH.into()), r)
    }
}
