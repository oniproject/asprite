use sdl2::event::Event;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::keyboard::Keycode;
use sdl2::render::{self, BlendMode, Texture, TextureCreator, TextureQuery, WindowCanvas};
use sdl2::{init, Sdl, VideoSubsystem, EventPump};
use sdl2::surface::SurfaceContext;
use sdl2::video::WindowContext;
use sdl2::ttf::{self, Sdl2TtfContext};
use sdl2::gfx::primitives::{DrawRenderer, ToColor};
use sdl2::rect;
use sdl2::image::LoadTexture;

use std::path::Path;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::cell::RefCell;

use specs::prelude::*;
use specs::world;
use specs::shred::MetaTable;

use ui;
use math::*;

use line::*;
use math::*;
use ui::Graphics;

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
        use line::*;
        world.register::<Line>();
        world.add_resource::<bool>(false);
    }
}

pub trait Painter {
    fn paint(&self, &mut WindowCanvas);
}

pub struct Canvas {
    sdl: Sdl,
    video: VideoSubsystem,
    pub canvas: RefCell<WindowCanvas>,
    events: EventPump,
    //font: ttf::Font<'static, 'static>,
    texture_creator: TextureCreator<WindowContext>,
    textures: HashMap<usize, (Texture, u32, u32)>,
    liner: Liner,

    pub hovered: AtomicBool,
    last_texture_id: AtomicUsize,

    mouse: ui::Mouse,

    vec: RefCell<Vec<Texture>>,

    cursors: Cursors,
}

impl Canvas {
    pub fn new(title: &str, w: u32, h: u32) -> Self {
        let sdl = init().unwrap();
        let video = sdl.video().unwrap();
        let window = video.window(title, w, h)
            .position_centered()
            //.opengl()
            .build()
            .unwrap();

        //let ttf = Box::leak(Box::new(ttf::init().unwrap()));
        //let font = ttf.load_font(font, 12).unwrap();

        let mut canvas = window.into_canvas()
            .software()
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
            //font,

            last_texture_id: AtomicUsize::new(0),
            texture_creator,
            textures: HashMap::new(),

            liner: Liner::new(),

            mouse: ui::Mouse::new(),

            vec: RefCell::new(Vec::new()),

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

    pub fn vline(&self, x: i16, y1: i16, y2: i16, color: u32) {
        self.canvas.borrow_mut().vline(x, y1, y2, color).unwrap();
    }

    pub fn hline(&self, x1: i16, x2: i16, y: i16, color: u32) {
        self.canvas.borrow_mut().hline(x1, x2, y, color).unwrap();
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

        self.canvas.borrow_mut().copy(texture, src, dst).unwrap();
    }
}

impl<'a> System<'a> for Canvas {
    type SystemData = (
        Write<'a, bool>,
        Write<'a, Option<::editor::App>>,
        WriteStorage<'a, Line>,
        Entities<'a>,
    );

    fn run(&mut self, (mut quit, mut app, mut lines, entities): Self::SystemData) {

        /*
        if let Some(ref mut app) = &mut *app {
            if let Some(event) = self.events.wait_event_timeout(30) {
                app.event(event.clone());
                for event in self.events.poll_iter() {
                    app.event(event.clone());
                }
            }
        }
        */

        {
            use std::iter;
            let poll = self.events.wait_event_timeout(60)
                .into_iter()
                .chain(self.events.poll_iter());

            for event in poll {
                if let Some(ref mut app) = &mut *app {
                    app.event(event.clone());
                }
                match event {
                    Event::Quit {..} => {
                        *quit = true;
                        return;
                    }

                    Event::KeyDown {keycode: Some(keycode), ..} => {
                        if keycode == Keycode::Escape {
                            *quit = true;
                            return;
                        } else if keycode == Keycode::Space {
                            println!("space down");
                            /*
                            for i in 0..400 {
                                self.canvas.pixel(i as i16, i as i16, 0xFF000FFu32).unwrap();
                            }
                            self.canvas.present();
                            */
                        }
                    }

                    Event::MouseButtonDown { x, y, .. } => {
                        let (x, y) = (x as i16, y as i16);
                        /*
                        for b in (&mut btns).join() {
                            b.mouse((x, y), Some(true))
                        }
                        */
                        self.liner.down(x, y);

                        self.mouse.cursor = Point2::new(x as f32, y as f32);
                        self.mouse.pressed[0] = true;
                    }
                    Event::MouseButtonUp {x, y, ..} => {
                        let (x, y) = (x as i16, y as i16);
                        /*
                        for b in (&mut btns).join() {
                            b.mouse((x, y), Some(false))
                        }
                        */
                        self.liner.up(x, y);

                        self.mouse.cursor = Point2::new(x as f32, y as f32);
                        self.mouse.released[0] = true;
                    }
                    Event::MouseMotion { x, y, .. } => {
                        self.mouse.cursor = Point2::new(x as f32, y as f32);

                        let (x, y) = (x as i16, y as i16);
                        /*
                        for b in (&mut btns).join() {
                            b.mouse((x, y), None)
                        }
                        */
                        self.liner.mov(x, y);

                        if let Some((start, end)) = self.liner.next() {
                            if start == end {
                                println!("dup");
                                return;
                            }

                            let r = end.0 as u8;
                            let g = end.1 as u8;
                            let color = Color::RGB(r, g, 255);

                            let entry = entities.create();
                            lines.insert(entry, Line {
                                start, end, color,
                            }).unwrap();
                        }
                    }
                    _ => {}
                }
            }
        }



        let cur = if self.hovered.load(Ordering::Relaxed) {
            SystemCursor::Hand
        } else {
            SystemCursor::Crosshair
        };
        self.cursors[&cur].set();
        self.hovered.store(false, Ordering::Relaxed);

        /*
        {
            let canvas = self.canvas.get_mut();
            canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
            canvas.clear();
        }

        for l in lines.join() {
            l.paint(self)
        }

        for b in btns.join() {
            b.paint(self)
        }
        */

        {
            let m = self.mouse;
            if let Some(ref mut app) = &mut *app {
                app.draw_ui(self, m);
            }
            self.mouse.cleanup();
            self.canvas.get_mut().present();
        }

        self.vec.get_mut().clear();

    }
}

/*
// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (::SCREEN_WIDTH as i32 - w) / 2;
    let cy = (::SCREEN_HEIGHT as i32 - h) / 2;
    rect!(cx, cy, w, h)
}
*/

impl Graphics for Canvas {
    type Texture = usize;
    type Color = u32;

    fn texture_dimensions(&self, id: &Self::Texture) -> Vector2<f32> {
        let (_, w, h) = self.textures[&id];
        Vector2::new(w as f32, h as f32)
    }

    fn quad(&self, color: Self::Color, rect: &Rect<f32>) {
        let sx = rect.min.x as i16;
        let sy = rect.min.y as i16;
        let ex = rect.max.x as i16 - 1;
        let ey = rect.max.y as i16 - 1;
        let color = Self::color(color);
        self.canvas.borrow_mut().box_(sx, sy, ex, ey, color)
            .expect("draw_box");
    }

    fn texture(&self, id: &Self::Texture, rect: &Rect<f32>) {
        let (ref texture, w, h) = self.textures[&id];
        let src = rect!(0, 0, w, h);
        let dst = rect!(rect.min.x, rect.min.y, rect.dx(), rect.dy());
        self.canvas.borrow_mut()
            .copy(texture, src, dst)
            .expect("copy texture");
    }

    fn texture_frame(&self, id: &Self::Texture, rect: &Rect<f32>, frame: &Rect<f32>) {
        let (ref texture, _, _) = self.textures[&id];
        let src = rect!(frame.min.x, frame.min.y, frame.dx(), frame.dy());
        let dst = rect!(rect.min.x, rect.min.y, rect.dx(), rect.dy());
        self.canvas.borrow_mut()
            .copy(texture, src, dst)
            .expect("copy texture");
    }

    fn measure_text(&self, text: &str) -> Vector2<f32> {
        //let (w, h) = self.font.size_of(text).expect("measure");
        let (w, h) = ::util::measure_text(text);
        Vector2::new(w as f32, h as f32)
    }

    fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
        /*
        let (r, g, b, a) = color.as_rgba();
        let color = Color::RGBA(r, g, b, a);

        let wh = self.measure_text(text);

        let surface = self.font.render(text)
            .blended(color)
            .expect("blended");

        let texture = self.texture_creator.create_texture_from_surface(&surface)
            .expect("create texture");

        let TextureQuery { width, height, .. } = texture.query();
        let target = rect!(base.x + 1.0, base.y + 1.0, width, height);

        /*
        let x0 = base.x as i16;
        let x1 = (base.x + wh.x) as i16;

        let y0 = base.y as i16;
        let y1 = (base.y + wh.y) as i16;

        let vx = [x0, x1, x1, x0];
        let vy = [y0, y0, y1, y1];
        */

        self.canvas.borrow_mut()
            .copy(&texture, None, Some(target))
            .expect("copy texture");
        //self.vec.borrow_mut().push(texture)
        */

        ::util::draw_text(&mut *self.canvas.borrow_mut(), base.x as i16, base.y as i16, text, color).unwrap();
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
