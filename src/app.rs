use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::gfx::primitives::DrawRenderer;

use prev::Prev;

use math::{Rect, Point2, Vector2};
use ui;
use ui::*;

use tool::{
    Input, Tool,
    EyeDropper, Bucket, Primitive, PrimitiveMode, Freehand,
    Brush, PreviewContext,
    ImageCell, image_cell, Editor,
    Receiver,
};

use render::{self, Canvas, TextureCanvas};
use draw::{blit, CanvasRead, CanvasWrite, Bounded, Palette};

use theme::*;
use grid::Grid;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentTool {
    Freehand,
    Bucket,
    EyeDropper,
    Primitive(PrimitiveMode),
}


pub struct App {
    pub init: bool,
    pub update: bool,
    pub quit: bool,

    pub files: Vec<ImageCell>,

    pub menubar: MenuBarModel,
    pub state: UiState,
    pub ui_mouse: ui::Mouse,

    // from tools

    pub current: CurrentTool,
    pub editor: Editor,

    pub freehand: Freehand<i32, u8>,
    pub prim: Primitive<i32, u8>,
    pub bucket: Bucket<i32, u8>,
    pub dropper: EyeDropper<i32, u8>,

    pub pos: Point2<i32>,
    pub grid: Grid,

    pub drag: bool,

    pub m: Point2<i32>,
    pub ed_mouse: Point2<i32>,
    pub zoom: i32,

    pub created: bool,
}

impl App {
    pub fn new(sprite: Receiver) -> Self {
        use tool::Context;
        let files = vec![image_cell(sprite)];

        let zoom = 1;
        let pos = Point2::new(300, 200);
        let sprite = files[0].clone();

        let mut editor = Editor::new(sprite);
        editor.sync();

        Self {
            init: false,

            update: true,
            quit: false,
            files,
            menubar: MenuBarModel { open_root: None },

            ui_mouse: ui::Mouse::new(),
            state: UiState::new(),

            zoom, pos,
            ed_mouse: Point2::new(-100, -100),
            m: Point2::new(-100, -100),
            drag: false,

            grid: Grid {
                show: true,
                size: Vector2::new(16, 16),
                offset: Vector2::new(-6, -6),
            },

            current: CurrentTool::Freehand,

            prim: Primitive::new(),
            bucket: Bucket::new(),
            freehand: Freehand::new(),
            dropper: EyeDropper::new(),

            editor,

            created: false,
        }
    }

    pub fn recreate(&mut self, m: ImageCell) {
        use tool::Context;
        self.editor.image = m;
        self.editor.sync();
        self.created = false;
    }

    pub fn input(&mut self, ev: Input<i32>) {
        if self.editor.sprite().as_receiver().is_lock() {
            return
        }
        match self.current {
            CurrentTool::Freehand => self.freehand.run(ev, &mut self.editor),
            CurrentTool::Bucket => self.bucket.run(ev, &mut self.editor),
            CurrentTool::EyeDropper => self.dropper.run(ev, &mut self.editor),
            CurrentTool::Primitive(mode) => {
                self.prim.mode = mode;
                self.prim.run(ev, &mut self.editor)
            }
        }
    }

    pub fn mouse_press(&mut self, p: Point2<i32>) {
        let p = self.set_mouse(p);
        if p.x >= 0 && p.y >= 0 {
            self.input(Input::Press(p));
        }
    }

    pub fn mouse_release(&mut self, p: Point2<i32>) {
        let p = self.set_mouse(p);
        if p.x >= 0 && p.y >= 0 {
            self.input(Input::Release(p));
        }
    }

    pub fn mouse_move(&mut self, p: Point2<i32>, v: Vector2<i32>) {
        let p = self.set_mouse(p);
        if self.drag {
            self.pos += v;
        } else {
            self.input(Input::Move(p));
        }
    }

    fn set_mouse(&mut self, p: Point2<i32>) -> Point2<i32> {
        let v = (p - self.pos) / self.zoom;
        self.ed_mouse = Point2::new(0, 0) + v;
        self.ed_mouse
    }

    pub fn zoom_from_center(&mut self, y: i32) {
        let v = self.editor.size();
        self.zoom(y, |diff| v * diff / 2);
    }

    pub fn zoom_from_mouse(&mut self, y: i32) {
        let p = self.ed_mouse;
        let v = Vector2::new(p.x, p.y);
        self.zoom(y, |diff| v * diff);
    }

    fn zoom<F: FnOnce(i32) -> Vector2<i32>>(&mut self, y: i32, f: F) {
        let last = self.zoom;
        self.zoom += y;
        if self.zoom < 1 { self.zoom = 1 }
        if self.zoom > 16 { self.zoom = 16 }
        let diff = last - self.zoom;

        let p = f(diff);

        self.pos.x += p.x;
        self.pos.y += p.y;
    }

    pub fn color(&self) -> u32 {
        let m = self.editor.sprite();
        let m = m.as_receiver();
        m.palette[m.color.get()]
    }

    pub fn pal(&self, color: u8) -> u32 {
        let m = self.editor.sprite();
        m.as_receiver().palette[color]
    }

    pub fn color_index(&self) -> u8 {
        let m = self.editor.sprite();
        m.as_receiver().color.get()
    }


    pub fn event(&mut self, event: sdl2::event::Event) {
        use tool::*;

        self.update = true;
        match event {
        Event::MouseMotion {x, y, xrel, yrel, ..} => {
            let p = Point2::new(x as i32, y as i32);
            let v = Vector2::new(xrel as i32, yrel as i32);
            self.mouse_move(p, v);
            self.ui_mouse.cursor = Point2::new(x as f32, y as f32);
        }

        Event::Quit {..} => self.quit = true,

        Event::KeyUp { keycode: Some(keycode), .. } => {
            match keycode {
                Keycode::LShift |
                Keycode::RShift =>
                    self.input(Input::Special(false)),
                Keycode::LCtrl |
                Keycode::RCtrl =>
                    self.drag = false,
                _ => (),
            }
        }

        Event::KeyDown { keycode: Some(keycode), keymod, ..} => {
            let shift = keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD);
            let _alt = keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD);
            let _ctrl = keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD);
            match keycode {
                Keycode::Escape => self.input(Input::Cancel),

                Keycode::Plus  | Keycode::KpPlus  => self.zoom_from_center(1),
                Keycode::Minus | Keycode::KpMinus => self.zoom_from_center(-1),

                Keycode::LShift |
                Keycode::RShift =>
                    self.input(Input::Special(true)),

                Keycode::LCtrl |
                Keycode::RCtrl =>
                    self.drag = true,

                Keycode::U => self.editor.undo(),
                Keycode::R => self.editor.redo(),

                //Keycode::Tab if shift => render.key = Some(gui::Key::PrevWidget),
                //Keycode::Tab if !shift => render.key = Some(gui::Key::NextWidget),

                _ => (),
            }
        }

        Event::MouseButtonDown { mouse_btn: MouseButton::Middle, .. } => {
            self.drag = true;
        }
        Event::MouseButtonUp { mouse_btn: MouseButton::Middle, .. } => {
            self.drag = false;
        }

        Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
            let p = Point2::new(x as i32, y as i32);
            self.mouse_press(p);
            self.ui_mouse.cursor = Point2::new(x as f32, y as f32);
            self.ui_mouse.pressed[0] = true;
        }
        Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
            let p = Point2::new(x as i32, y as i32);
            self.mouse_release(p);
            self.ui_mouse.cursor = Point2::new(x as f32, y as f32);
            self.ui_mouse.released[0] = true;
        }

        Event::MouseWheel { y, ..} => { self.zoom_from_mouse(y as i32); }

        _ => (),
        }
    }

    pub fn paint_sprites(&mut self, render: &mut render::Canvas) {
        if let Some(r) = self.editor.take_redraw() {
            render.canvas(EDITOR_SPRITE_ID, |canvas: &mut TextureCanvas, _w, _h| {
                let r = r.normalize();
                let clear_rect = rect!(r.min.x, r.min.y, r.dx(), r.dy());
                //canvas.set_clip_rect(r);

                let clear_color = color!(TRANSPARENT);
                // XXX let clear_color = color!(self.pal(0));
                canvas.set_draw_color(clear_color);
                canvas.draw_rect(clear_rect).unwrap();

                canvas.clear();

                self.editor.draw_pages(|page, palette| {
                    let transparent = page.transparent;
                    let br = Rect::from_coords_and_size(0, 0, page.width as i32, page.height as i32);
                    let r = br;
                    blit(r, br, &page.page, |x, y, color| {
                        let c = if Some(color) != transparent {
                            palette[color].to_be()
                        } else {
                            TRANSPARENT
                        };
                        canvas.pixel(x as i16, y as i16, c).unwrap();
                    })
                });
            });
        }

        render.canvas(EDITOR_PREVIEW_ID, |canvas: &mut TextureCanvas, w, h| {
            canvas.set_draw_color(color!(TRANSPARENT));
            canvas.clear();

            let m = self.editor.sprite();
            let mut prev = Prev {
                canvas,
                rect: Rect::from_coords_and_size(0, 0, w as i32, h as i32),
                palette: &m.as_receiver().palette,
                editor: &self.editor,
            };

            match self.current {
            CurrentTool::Freehand => self.freehand.preview(self.ed_mouse, &mut prev),
            _ => (),
            }
        });

        let pos = Point2::new(self.pos.x as i16, self.pos.y as i16);
        let zoom = self.zoom as i16;

        {
            use ui::Graphics;

            const SIZE: u32 = 10;
            let (w, h) = render.size();
            let max_x = w / SIZE + 1;
            let max_y = h / SIZE + 1;

            // draw back grid
            for y in 0..max_y {
                for x in 0..max_x {
                    let is = x % 2 == y % 2;
                    let x = SIZE * x;
                    let y = SIZE * y;
                    let min = Point2::new(x as f32, y as f32);
                    let dim = Vector2::new(SIZE as f32, SIZE as f32);
                    let r = Rect::from_min_dim(min, dim);
                    if is {
                        render.quad(0x333333_FFu32.to_be(), &r);
                    } else {
                        render.quad(0x000000_FFu32.to_be(), &r);
                    }
                }
            }
        }

        render.image_zoomed(EDITOR_SPRITE_ID, pos, zoom);
        render.image_zoomed(EDITOR_PREVIEW_ID, pos, zoom);

        self.grid.paint(render, zoom, Rect {
            min: self.pos,
            max: self.pos + self.editor.size(),
        });
    }

    pub fn paint(&mut self, canvas: &mut Canvas) {
        if !self.init {
            use tool::Context;
            self.init = true;
            self.editor.change_color(2);
            canvas.load_texture(ICON_TOOL_FREEHAND, "res/tool_freehand.png");
            canvas.load_texture(ICON_TOOL_PIP, "res/tool_pip.png");
            canvas.load_texture(ICON_TOOL_RECT, "res/tool_rect.png");
            canvas.load_texture(ICON_TOOL_CIRC, "res/tool_circ.png");
            canvas.load_texture(ICON_TOOL_FILL, "res/tool_fill.png");

            canvas.load_texture(ICON_UNDO, "res/undo.png");
            canvas.load_texture(ICON_REDO, "res/redo.png");
        }

        if !self.created {
            self.created = true;
            let m = self.editor.sprite();
            let m = m.as_receiver();
            let (w, h) = (m.width as u32, m.height as u32);
            canvas.create_texture(EDITOR_SPRITE_ID, w, h);
            canvas.create_texture(EDITOR_PREVIEW_ID, w, h);
        }

        self.paint_sprites(canvas);

        let (w, h) = canvas.canvas.borrow().logical_size();
        let wh = Vector2::new(w as f32, h as f32);

        let rect = Rect::from_min_dim(Point2::new(0.0, 0.0), wh);
        let ctx = Context::new(canvas, rect, self.ui_mouse);
        self.ui_mouse.cleanup();

        let widgets = [
            Flow::with_height(MENUBAR_HEIGHT).expand_across(),
            Flow::with_height(TOOLBAR_HEIGHT).expand_across(),
            Flow::auto(1.0),
            Flow::with_height(200.0).expand_across(),
            Flow::with_height(STATUSBAR_HEIGHT).expand_across(),
        ];

        let colors = [
            MENUBAR_BG,
            TOOLBAR_BG,
            rgba(0),
            rgba(0x3a4351_FF),
            STATUSBAR_BG,
        ];

        let mut iter = ctx.vertical_flow(0.0, 0.0, &widgets)
            .zip(colors.iter().cloned())
            .map(|(ctx, color)| {
                ctx.quad(color, &ctx.rect());
                ctx
            });

        self.menubar(&iter.next().unwrap());
        self.toolbar(&iter.next().unwrap());
        self.content(&iter.next().unwrap());
        iter.next().unwrap();
        self.statusbar(&iter.next().unwrap());
        if false { self.sample(&ctx) }
        self.second_menubar(&ctx);
    }

    fn content(&mut self, ctx: &ui::Context<Canvas>) {
        use tool::Context;
        let mut r = ctx.rect();
        r.max.x = r.min.x + 250.0;

        let ctx = ctx.sub_rect(r);
        {
            let r = ctx.rect();
            let start = r.min;

            const WH: usize = 15;
            let w = (r.dx() as usize - 5 * 2) / WH;
            for i in 0..256 {
                let r = Rect::from_min_dim(start + Vector2::new(
                    ( 5 + (i % w) * WH) as f32,
                    (40 + (i / w) * WH) as f32,
                ), Vector2::new(WH as f32, WH as f32));
                let color = self.pal(i as u8);

                if BTN.behavior(&ctx.sub_rect(r), &mut self.state, &mut ()) {
                    self.editor.change_color(i as u8);
                }
                ctx.quad(rgba(color), &r.pad(1.0));
            }
        }
    }

    fn statusbar(&mut self, ctx: &ui::Context<Canvas>) {
        let text = format!("zoom: {}  #{:<3}", self.zoom, self.color_index());
        ctx.label(0.01, 0.5, WHITE, &text);
        let text = format!("[{:>+5} {:<+5}]", self.ed_mouse.x, self.ed_mouse.y);
        ctx.label(0.2, 0.5, WHITE, &text);
    }

    fn sample(&mut self, ctx: &ui::Context<Canvas>) {
        let widgets = &[
            Flow::with_wh(60.0, 40.0),
            Flow::with_wh(60.0, 40.0),
            Flow::with_wh(60.0, 40.0).along_weight(1.0).expand_along(),
            Flow::with_wh(40.0, 40.0),
            Flow::with_wh(40.0, 40.0).expand_across(),
        ];

        let ctx = {
            let anchor = Rect {
                min: Point2::new(0.25, 0.25),
                max: Point2::new(0.75, 0.75),
            };
            let offset = Rect {
                min: Point2::new(0.0, 0.0),
                max: Point2::new(0.0, 0.0),
            };

            ctx.sub().transform(anchor, offset).build()
        };

        //println!("{:?}", ctx.rect());

        ctx.quad(rgba(0xFF0000_CC), &ctx.rect());

        static mut TOGGLE_STATE: bool = false;

        static mut SLIDER_H: SliderModel = SliderModel {
            min: 1.0,
            current: 2.7,
            max: 3.0,
        };

        static mut SLIDER_V: SliderModel = SliderModel {
            min: 1.0,
            current: 2.7,
            max: 3.0,
        };

        let toggle_state = unsafe { &mut TOGGLE_STATE };
        let slider_state_h = unsafe { &mut SLIDER_H };
        let slider_state_v = unsafe { &mut SLIDER_V };

        let mut i = 0;
        for ctx in ctx.horizontal_flow(0.0, 0.0, widgets) {
            match i {
            1 => {
                TOGGLE.behavior(&ctx, &mut self.state, toggle_state);
                ctx.label(0.5, 0.5, WHITE, &format!("tgl{}", i));
            }
            2 => {
                HSLIDER.behavior(&ctx, &mut self.state, slider_state_h);
                ctx.label(0.5, 0.5, WHITE, &format!("val{}: {}", i, slider_state_h.current));
            }
            4 => {
                VSLIDER.behavior(&ctx, &mut self.state, slider_state_v);
                ctx.label(0.5, 0.5, WHITE, &format!("val{}: {}", i, slider_state_v.current));
            }
            _ => {
                if BTN.behavior(&ctx, &mut self.state, &mut ()) {
                    println!("{} click", i);
                }
                ctx.label(0.5, 0.5, WHITE, &format!("btn{}", i));
            }
            }

            i += 1;
        }
    }

    fn toolbar(&mut self, ctx: &ui::Context<Canvas>) {
        let height = ctx.rect().dy();
        let btn = Flow::with_wh(height, height);
        let flow = [
            btn, btn,
            Flow::auto(1.0),
            btn, btn, btn, btn, btn,
            Flow::auto(1.0),
            Flow::with_width(130.0).expand_across(),
            Flow::auto(1.0),
        ];
        let mut flow = ctx.horizontal_flow(0.0, 0.5, &flow);

        {
            let undo = flow.next().unwrap();
            let redo = flow.next().unwrap();

            if BTN.behavior(&redo, &mut self.state, &mut ()) {
                self.editor.redo();
            }
            if BTN.behavior(&undo, &mut self.state, &mut ()) {
                self.editor.undo();
            }

            ctx.draw().texture(&ICON_UNDO, &undo.rect());
            ctx.draw().texture(&ICON_REDO, &redo.rect());
        }

        flow.next().unwrap();

        {
            static MODES: &[(usize, CurrentTool)] = &[
                (ICON_TOOL_FREEHAND, CurrentTool::Freehand),
                (ICON_TOOL_FILL, CurrentTool::Bucket),
                (ICON_TOOL_CIRC, CurrentTool::Primitive(PrimitiveMode::Ellipse)),
                (ICON_TOOL_RECT, CurrentTool::Primitive(PrimitiveMode::Rect)),
                (ICON_TOOL_PIP, CurrentTool::EyeDropper),
            ];

            for ((icon, tool), ctx) in MODES.iter().cloned().zip(flow.by_ref()) {
                if BTN.behavior(&ctx, &mut self.state, &mut ()) {
                    self.current = tool;
                }
                let r = ctx.rect();
                if self.current == tool {
                    BTN.pressed.draw_frame(ctx.draw(), ctx.rect());
                }
                ctx.draw().texture(&icon, &r);
            }
        }

        let _ = flow.next().unwrap();
        let ctx = flow.next().unwrap();

        match ::layout::edit_num(&ctx, &mut self.state, self.zoom, "zoom") {
            Some(true) => self.zoom_from_center(1),
            Some(false) => self.zoom_from_center(-1),
            _ => (),
        }
    }

    fn menubar(&mut self, ctx: &ui::Context<Canvas>) {
        MENUBAR.run(&ctx, &mut self.state, &mut self.menubar, &[
            (ctx.reserve_widget_id(), "File"),
            (ctx.reserve_widget_id(), "Edit"),
            (ctx.reserve_widget_id(), "View"),
            (ctx.reserve_widget_id(), "Tools"),
            (ctx.reserve_widget_id(), "Help"),
        ]);
    }

    fn second_menubar(&mut self, ctx: &ui::Context<Canvas>) {
        let items = [
            Item::Text(Command::New, "New", "Ctrl-N"),
            Item::Text(Command::Open, "Open", "Ctrl-O"),
            Item::Text(Command::Recent, "Recent", ">"),
            Item::Separator,
            Item::Text(Command::Save, "Save", "Ctrl-S"),
            Item::Text(Command::SaveAs, "Save as...", "Shift-Ctrl-S"),
            Item::Separator,
            Item::Text(Command::Quit, "Quit", "Ctrl-Q"),
        ];

        if let Some((id, base_rect)) = self.menubar.open_root {
            let mut exit = true;
            match MENU.run(&ctx, &mut self.state, id, base_rect, &items) {
                MenuEvent::Nothing => exit = false,
                MenuEvent::Exit => (),
                MenuEvent::Clicked(Command::Open) => {
                    println!("open_file: {:?}", ::open::open_file());
                }
                MenuEvent::Clicked(id) => {
                    println!("click: {:?}", id);
                }
            }
            if exit {
                self.menubar.open_root = None;
            }
        }
    }
}
