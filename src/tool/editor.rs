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

pub type ImageCell = Arc<Mutex<Record<Receiver, DrawCommand>>>;
pub fn image_cell(sprite: Receiver) -> ImageCell {
    Arc::new(Mutex::new(Record::new(sprite)))
}

pub struct Editor {
    pub image: ImageCell,

    canvas: Frame,
    start: usize,
    stride: usize,
}

impl Editor {
    pub fn new(pos: Point2<i32>, image: ImageCell) -> Self {
        let (w, h) = {
            let m = image.lock().unwrap();
            let m = m.as_receiver();
            (m.width, m.height)
        };
        Self {
            image,

            canvas: Frame::new(w, h),
            start: 0,
            stride: w,
        }
    }

    pub fn zoom(&self) -> i32 {
        self.sprite().as_receiver().zoom
    }

    pub fn pos(&self) -> Point2<i32> {
        self.sprite().as_receiver().pos
    }

    pub fn size(&self) -> Vector2<i32> {
        let m = self.sprite();
        let m = m.as_receiver();
        Vector2::new(m.width as i32, m.height as i32)
    }

    pub fn rect(&self) -> Rect<i32> {
        self.sprite().as_receiver().rect()
    }

    pub fn take_created(&mut self) -> bool {
        let mut m = self.sprite();
        let m = m.as_mut_receiver();

        let c = m.created;
        m.created = true;
        c
    }

    pub fn color(&self) -> u32 {
        let m = self.sprite();
        let m = m.as_receiver();
        m.palette[m.color.get()]
    }

    pub fn pal(&self, color: u8) -> u32 {
        let m = self.sprite();
        m.as_receiver().palette[color]
    }

    pub fn color_index(&self) -> u8 {
        let m = self.sprite();
        m.as_receiver().color.get()
    }

    pub fn take_redraw(&mut self) -> Option<Rect<i32>> {
        let mut m = self.sprite();
        m.as_mut_receiver().take_update()
    }

    pub fn sprite(&self) -> MutexGuard<Record<Receiver, DrawCommand>> {
        self.image.lock().unwrap()
    }

    pub fn redo(&mut self) {
        use super::Context;
        self.image.lock().unwrap().redo();
        self.sync();
    }

    pub fn undo(&mut self) {
        use super::Context;
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
        self.sprite().as_mut_receiver().update(r)
    }

    fn sync(&mut self) {
        let mut m = self.image.lock().unwrap();
        let mut m = m.as_mut_receiver();
        let (layer, frame) = {
            (m.layer.get(), m.frame.get())
        };
        self.canvas.copy_from(&m.page(layer, frame));
        m.update_all();
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
