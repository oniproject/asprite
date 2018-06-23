#![allow(dead_code)]

use sdl2::{
    rect, init, Sdl, VideoSubsystem, EventPump,
    event::{Event, WindowEvent},
    pixels::{Color, PixelFormatEnum},
    render::{BlendMode, Texture, TextureCreator, TextureQuery, WindowCanvas},
    video::WindowContext,
    //ttf::{self, Sdl2TtfContext},
    gfx::primitives::{DrawRenderer, ToColor},
    image::LoadTexture,
};

use std::{
    path::Path,
    sync::atomic::{AtomicUsize, AtomicBool, Ordering},
    cell::RefCell,
};

use specs::{
    prelude::*,
    world,
};

use ui;
use math::*;

pub type TextureCanvas = WindowCanvas;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

use sdl2::mouse::{Cursor, SystemCursor};
use std::collections::HashMap;

pub type Cursors = HashMap<SystemCursor, Cursor>;
pub fn create_cursors() -> Cursors {
    let cursors = [
        SystemCursor::Arrow,
        SystemCursor::IBeam,
        SystemCursor::Wait,
        SystemCursor::Crosshair,
        SystemCursor::WaitArrow,
        SystemCursor::SizeNWSE,
        SystemCursor::SizeNESW,
        SystemCursor::SizeWE,
        SystemCursor::SizeNS,
        SystemCursor::SizeAll,
        SystemCursor::No,
        SystemCursor::Hand,
    ];

    let cursors: HashMap<_, _> = cursors.iter().map(|&c| (c, Cursor::from_system(c).unwrap())).collect();
    cursors[&SystemCursor::Crosshair].set();
    cursors
}


pub struct Bundle;

impl world::Bundle for Bundle {
    fn add_to_world(self, world: &mut World) {
        world.add_resource::<bool>(false);
    }
}

pub struct Canvas {
    sdl: Sdl,
    video: VideoSubsystem,
    canvas: RefCell<WindowCanvas>,
    events: EventPump,
    texture_creator: TextureCreator<WindowContext>,
    textures: HashMap<usize, (Texture, u32, u32)>,

    hovered: AtomicBool,
    last_texture_id: AtomicUsize,

    cursors: Cursors,
}

impl Canvas {
    pub fn new(title: &str, w: u32, h: u32) -> Self {
        let sdl = init().unwrap();
        let video = sdl.video().unwrap();
        let window = video.window(title, w, h)
            .position_centered()
            //.opengl()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas()
            //.software()
            //.present_vsync()
            .build().unwrap();

        let texture_creator = canvas.texture_creator();

        canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        canvas.clear();
        canvas.present();

        let events = sdl.event_pump().unwrap();

        Self {
            sdl,
            video,
            canvas: RefCell::new(canvas),
            events,

            last_texture_id: AtomicUsize::new(0),
            texture_creator,
            textures: HashMap::new(),

            hovered: AtomicBool::new(false),
            cursors: create_cursors(),
        }
    }

    fn color<C>(color: C) -> Color
        where C: ToColor
    {
        let (r, g, b, a) = color.as_rgba();
        Color::RGBA(r, g, b, a)
    }

    pub fn vline(&mut self, x: i16, y1: i16, y2: i16, color: u32) {
        self.canvas.get_mut().vline(x, y1, y2, color).unwrap();
    }

    pub fn hline(&mut self, x1: i16, x2: i16, y: i16, color: u32) {
        self.canvas.get_mut().hline(x1, x2, y, color).unwrap();
    }

    pub fn size(&self) -> (u32, u32) {
        self.canvas.borrow().logical_size()
    }

    fn gen_id<T: Into<Option<usize>>>(&mut self, id: T) -> usize {
        id.into().unwrap_or_else(|| self.last_texture_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn load_texture<T: Into<Option<usize>>, P: AsRef<Path>>(&mut self, id: T, filename: P) -> usize {
        let id = self.gen_id(id);
        let mut texture = self.texture_creator.load_texture(filename).unwrap();
        texture.set_blend_mode(BlendMode::Blend);
        let TextureQuery { width, height, .. } = texture.query();
        self.textures.insert(id, (texture, width, height));
        id
    }

    pub fn create_texture<T: Into<Option<usize>>>(&mut self, id: T, w: u32, h: u32) -> usize {
        let id = self.gen_id(id);
        let mut texture = self.texture_creator
            .create_texture_target(PixelFormatEnum::RGBA8888, w, h).unwrap();
        texture.set_blend_mode(BlendMode::Blend);
        let TextureQuery { width, height, .. } = texture.query();
        self.textures.insert(id, (texture, width, height));
        id
    }

    pub fn get_texture(&mut self, id: usize) -> (&mut Texture, u32, u32) {
        self.textures.get_mut(&id)
            .map(|(ref mut v, w, h)| (v, *w, *h))
            .unwrap()
    }

    pub fn canvas<F>(&mut self, id: usize, f: F)
        where F: FnOnce(&mut TextureCanvas, u32, u32)
    {
        let texture = self.textures.get_mut(&id);
        if let Some(texture) = texture {
            let w = texture.1;
            let h = texture.2;
            self.canvas.borrow_mut()
                .with_texture_canvas(&mut texture.0, |canvas| f(canvas, w, h)).unwrap();
        }
    }

    pub fn image_zoomed(&mut self, id: usize, pos: Point2<i16>, zoom: i16) {
        let (ref texture, w, h) = self.textures[&id];
        let (w, h) = (w as i16, h as i16);
        let src = rect!(0, 0, w, h);
        let dst = rect!(pos.x, pos.y, w * zoom, h * zoom);

        self.canvas.get_mut().copy(texture, src, dst).unwrap();
    }
}

impl<'a> System<'a> for Canvas {
    type SystemData = (
        Write<'a, bool>,
        WriteExpect<'a, ::app::App>,
    );

    fn run(&mut self, (mut quit, mut app): Self::SystemData) {
        {
            let poll = self.events.poll_iter();

            for event in poll {
                app.event(event.clone());
                if app.quit {
                    *quit = true;
                    return;
                }
                if let Event::Window { win_event: WindowEvent::Resized(w, h), .. } = event {
                    self.canvas.get_mut().set_logical_size(w as u32, h as u32).unwrap();
                }
            }
        }

        app.paint(self);
        self.canvas.get_mut().present();

        let cur = if self.hovered.load(Ordering::Relaxed) {
            SystemCursor::Hand
        } else {
            SystemCursor::Crosshair
        };
        self.cursors[&cur].set();
        self.hovered.store(false, Ordering::Relaxed);
    }
}

impl ui::Graphics for Canvas {
    type Texture = usize;
    type Color = u32;

    fn texture_dimensions(&self, id: Self::Texture) -> Vector2<f32> {
        let (_, w, h) = self.textures[&id];
        Vector2::new(w as f32, h as f32)
    }

    fn quad(&self, color: Self::Color, rect: Rect<f32>) {
        let sx = rect.min.x as i16;
        let sy = rect.min.y as i16;
        let ex = rect.max.x as i16 - 1;
        let ey = rect.max.y as i16 - 1;
        let color = Self::color(color);
        self.canvas.borrow_mut().box_(sx, sy, ex, ey, color)
            .expect("draw_box");
    }

    fn texture(&self, id: Self::Texture, rect: Rect<f32>) {
        let (ref texture, w, h) = self.textures[&id];
        let src = rect!(0, 0, w, h);
        let dst = rect!(rect.min.x, rect.min.y, rect.dx(), rect.dy());
        self.canvas.borrow_mut()
            .copy(texture, src, dst)
            .expect("copy texture");
    }

    fn texture_frame(&self, id: Self::Texture, rect: Rect<f32>, frame: Rect<f32>) {
        let (ref texture, _, _) = self.textures[&id];
        let src = rect!(frame.min.x, frame.min.y, frame.dx(), frame.dy());
        let dst = rect!(rect.min.x, rect.min.y, rect.dx(), rect.dy());
        self.canvas.borrow_mut()
            .copy(texture, src, dst)
            .expect("copy texture");
    }

    fn measure_text(&self, text: &str) -> Vector2<f32> {
        let (w, h) = measure_text(text);
        Vector2::new(w as f32, h as f32)
    }

    fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
        if let Err(err) = draw_text(&mut *self.canvas.borrow_mut(), base.x as i16, base.y as i16, text, color) {
            println!("error: {}", err);
        }
    }

    fn clip(&self, r: Rect<i16>) {
        self.canvas.borrow_mut().set_clip_rect(rect!(r.min.x, r.min.y, r.dx(), r.dy()));
    }

    fn unclip(&self) {
        self.canvas.borrow_mut().set_clip_rect(None);
    }

    fn set_hovered(&self) {
        self.hovered.store(true, Ordering::Relaxed);
    }
}

fn measure_text(text: &str) -> (i16, i16) {
    let w = text.len() as i16 * 8 + 0;
    let h = 8;
    (w, h)
}

fn draw_text<C: ToColor + Copy>(canvas: &mut WindowCanvas, x: i16, y: i16, label: &str, color: C) -> Result<(), String> {
    //let mut x = x + 1;
    //let y = y + 1;
    let mut x = x;
    let y = y;
    for c in label.chars() {
        canvas.character(x, y, c, color)?;
        x += 8;
    }
    Ok(())
}

