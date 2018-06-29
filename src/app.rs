use sdl2::{
    event::Event,
    keyboard::{self, Keycode},
    mouse::MouseButton,
};
use std::time::Instant;

use layout::{EditorLayout, edit_num};
use prev::Prev;

use math::{Rect, Point2, Vector2};
use ui;
use ui::*;

use tool::{
    Tool,
    EyeDropper, Bucket, Primitive, PrimitiveMode, Freehand,
    PreviewContext,
    Editor,
    Receiver,
};

use render::{self, Canvas};
use draw::{Shape, Bounded};

use theme::*;
use grid::Grid;


#[macro_export]
macro_rules! flow {
    (h $ctx:expr => { $($flow:expr => |$i:ident| $block:expr)+ }) => {
        let widgets = [
            $($flow),+
        ];
        let mut iter = $ctx.horizontal_flow(0.0, 0.0, &widgets);
        $({
            (|$i: ::ui::Context<Canvas>| $block)(iter.next().unwrap())
        });+
    };

    (v $ctx:expr => { $($flow:expr => |$i:ident| $block:expr)+ }) => {
        let widgets = [
            $($flow),+
        ];
        let mut iter = $ctx.vertical_flow(0.0, 0.0, &widgets);
        $({
            (|$i: ::ui::Context<Canvas>| $block)(iter.next().unwrap())
        });+
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentTool {
    Freehand,
    Bucket,
    EyeDropper,
    Primitive(PrimitiveMode),
}

macro_rules! tools {
    ($self: expr, $name: ident, $ev: expr) => {
        match $self.current {
            CurrentTool::Freehand => $self.freehand.$name($ev, &mut $self.editor),
            CurrentTool::Bucket => $self.bucket.$name($ev, &mut $self.editor),
            CurrentTool::EyeDropper => $self.dropper.$name($ev, &mut $self.editor),
            CurrentTool::Primitive(mode) => {
                $self.prim.mode = mode;
                $self.prim.$name($ev, &mut $self.editor)
            }
        }
    }
}

pub struct App {
    pub init: bool,
    pub quit: bool,

    pub menubar: MenuBarModel,
    pub state: UiState,
    pub ui_mouse: ui::Mouse,

    pub editor: Editor,
    pub grid: Grid,

    pub current: CurrentTool,

    pub freehand: Freehand<i32>,
    pub prim: Primitive<i32>,
    pub bucket: Bucket,
    pub dropper: EyeDropper,

    pub mouse: Point2<i32>,
    pub drag: bool,

    file_menu_id: ui::Id,
    brush_menu_id: ui::Id,

    rect: Rect<i32>,
    in_widget: bool,

    time: Instant,
    data: Vec<u8>,
}

impl App {
    pub fn new(sprite: Receiver) -> Self {
        use tool::Context;

        let mut editor = Editor::new(sprite);
        editor.sync();
        editor.image.as_mut_receiver().pos = Point2::new(300, 200);

        Self {
            init: false,

            quit: false,
            menubar: MenuBarModel { open_root: None },

            rect: Rect::default(),
            in_widget: false,

            ui_mouse: ui::Mouse::new(),
            state: UiState::new(),

            grid: Grid {
                visible: true,
                size: Vector2::new(16, 16),
                offset: Vector2::new(0, 0),
            },
            editor,
            current: CurrentTool::Freehand,
            prim: Primitive::new(),
            bucket: Bucket::new(),
            freehand: Freehand::new(),
            dropper: EyeDropper::new(),

            file_menu_id: ui::Id::from(0xDEAD_BEED),
            brush_menu_id: ui::Id::from(0xDEAD_BEED),

            mouse: Point2::new(-100, -100),
            drag: false,

            time: Instant::now(),
            data: Vec::new(),
        }
    }

    pub fn preview<T>(&self, mut prev: T)
        where T: PreviewContext<i32, u8>
    {
        match self.current {
            CurrentTool::Freehand => self.freehand.preview(self.mouse, &mut prev),
            CurrentTool::Bucket => self.bucket.preview(self.mouse, &mut prev),
            CurrentTool::EyeDropper => self.dropper.preview(self.mouse, &mut prev),
            CurrentTool::Primitive(_) => self.prim.preview(self.mouse, &mut prev),
        }
    }

    pub fn cancel(&mut self) {
        match self.current {
            CurrentTool::Freehand => self.freehand.cancel(&mut self.editor),
            CurrentTool::Bucket => self.bucket.cancel(&mut self.editor),
            CurrentTool::EyeDropper => self.dropper.cancel(&mut self.editor),
            CurrentTool::Primitive(mode) => {
                self.prim.mode = mode;
                self.prim.cancel(&mut self.editor);
            }
        }
    }

    pub fn special(&mut self, v: bool) {
        if self.editor.image.as_receiver().is_lock() { return }
        tools!(self, special, v)
    }

    pub fn mouse_press(&mut self, p: Point2<i32>) {
        if let Some(p) = self.set_mouse(p) {
            tools!(self, press, p);
        }
    }

    pub fn mouse_release(&mut self, p: Point2<i32>) {
        if let Some(p) = self.set_mouse(p) {
            tools!(self, release, p);
        }
    }

    pub fn mouse_move(&mut self, p: Point2<i32>, v: Vector2<i32>) {
        if self.drag {
            self.editor.image.as_mut_receiver().pos += v;
        } else if let Some(p) = self.set_mouse(p) {
            tools!(self, movement, p);
        }
    }

    fn set_mouse(&mut self, p: Point2<i32>) -> Option<Point2<i32>> {
        let m = self.editor.image.as_receiver();
        if !m.is_lock() && self.rect.contains(p) {
            let v = (p - self.editor.pos()) / self.editor.zoom();
            let p = Point2::new(0, 0) + v;
            self.mouse = p;
            self.in_widget = true;
            if m.bounds().contains(p) {
                return Some(self.mouse)
            }
        } else {
            self.in_widget = false;
        }
        None
    }

    pub fn zoom_from_center(&mut self, y: i32) {
        let v = self.editor.size();
        self.editor.image
             .as_mut_receiver()
             .zoom(y, |diff| v * diff / 2);
    }

    pub fn zoom_from_mouse(&mut self, y: i32) {
        let v = Vector2::new(self.mouse.x, self.mouse.y);
        self.editor.image
            .as_mut_receiver()
            .zoom(y, |diff| v * diff);
    }

    pub fn event(&mut self, event: Event) {
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
                Keycode::RShift => self.special(false),
                Keycode::LCtrl |
                Keycode::RCtrl => self.drag = false,
                _ => (),
            }
        }

        Event::KeyDown { keycode: Some(keycode), keymod, ..} => {
            let _shift = keymod.intersects(keyboard::LSHIFTMOD | keyboard::RSHIFTMOD);
            let _alt = keymod.intersects(keyboard::LALTMOD | keyboard::RALTMOD);
            let _ctrl = keymod.intersects(keyboard::LCTRLMOD | keyboard::RCTRLMOD);
            match keycode {
                Keycode::Escape => self.cancel(),

                Keycode::Plus  | Keycode::KpPlus  => self.zoom_from_center(1),
                Keycode::Minus | Keycode::KpMinus => self.zoom_from_center(-1),

                Keycode::LShift |
                Keycode::RShift => self.special(true),
                Keycode::LCtrl |
                Keycode::RCtrl => self.drag = true,

                Keycode::U => self.editor.undo(),
                Keycode::R => self.editor.redo(),

                //Keycode::Tab if shift => render.key = Some(gui::Key::PrevWidget),
                //Keycode::Tab if !shift => render.key = Some(gui::Key::NextWidget),

                _ => (),
            }
        }

        Event::MouseButtonDown { mouse_btn: MouseButton::Middle, .. } => self.drag = true,
        Event::MouseButtonUp   { mouse_btn: MouseButton::Middle, .. } => self.drag = false,

        Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
            let p = Point2::new(x as i32, y as i32);
            self.mouse_press(p);
            self.ui_mouse.cursor = Point2::new(x as f32, y as f32);
            self.ui_mouse.pressed = true;
        }
        Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
            let p = Point2::new(x as i32, y as i32);
            self.mouse_release(p);
            self.ui_mouse.cursor = Point2::new(x as f32, y as f32);
            self.ui_mouse.released = true;
        }

        Event::MouseWheel { y, ..} => { self.zoom_from_mouse(y as i32); }

        _ => (),
        }
    }

    pub fn paint_sprites(&mut self, render: &mut render::Canvas) {
        let (t, w, h) = render.get_texture(EDITOR_SPRITE_ID);

        self.data.clear();
        let size = w * h * 4;
        self.data.resize(size as usize, 0u8);

        let ptr = self.data.as_mut_ptr();
        self.editor.draw_pages(|page, palette| {
            let transparent = page.transparent;
            let mut ptr = ptr;
            for &c in &page.page {
                unsafe {
                    if Some(c) != transparent {
                        let c = palette[c].to_le();
                        *ptr.add(0) = ( c        & 0xFF) as u8;
                        *ptr.add(1) = ((c >>  8) & 0xFF) as u8;
                        *ptr.add(2) = ((c >> 16) & 0xFF) as u8;
                        *ptr.add(3) = ((c >> 24) & 0xFF) as u8;
                    }
                    ptr = ptr.add(4);
                }
            }
        });

        let ptr = self.data.as_mut_ptr();
        if self.in_widget {
            self.preview(Prev {
                ptr,
                rect: Rect::from_coords_and_size(0, 0, w as i32, h as i32),
                editor: &self.editor,
            });
        }
        t.update(None, &self.data, self.editor.size().x as usize * 4).unwrap();
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

            canvas.load_texture(ICON_EYE, "res/eye.png");
            canvas.load_texture(ICON_EYE_OFF, "res/eye_off.png");

            canvas.load_texture(ICON_LOCK, "res/lock.png");
            canvas.load_texture(ICON_LOCK_OFF, "res/lock_off.png");

            canvas.load_texture(ICON_CHECK_ON, "res/check_on.png");
            canvas.load_texture(ICON_CHECK_OFF, "res/check_off.png");
        }

        if !self.editor.take_created() {
            let m = self.editor.image.as_receiver();
            let (w, h) = (m.width as u32, m.height as u32);
            canvas.create_texture(EDITOR_SPRITE_ID, w, h);
        }

        canvas.clip(self.rect.cast().unwrap());
        {
            use ui::Graphics;

            const SIZE: u32 = 10;
            let (w, h) = canvas.size();
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
                        canvas.quad(0x333333_FFu32.to_be(), r);
                    } else {
                        canvas.quad(0x000000_FFu32.to_be(), r);
                    }
                }
            }
        }

        {
            self.paint_sprites(canvas);
            let rect = self.editor.rect();
            let pos = Point2::new(rect.min.x as i16, rect.min.y as i16);
            let zoom = self.editor.zoom() as i16;
            canvas.image_zoomed(EDITOR_SPRITE_ID, pos, zoom);
            {
                self.grid.paint(canvas, zoom, rect);
            }
        }
        canvas.unclip();

        let (w, h) = canvas.size();
        let wh = Vector2::new(w as f32, h as f32);

        let rect = Rect::from_min_dim(Point2::new(0.0, 0.0), wh);
        let ctx = Context::new(canvas, rect, self.ui_mouse);
        self.ui_mouse.cleanup();

        flow!(v ctx => {
            Flow::with_height(MENUBAR_HEIGHT).expand_across()   => |ctx| { self.menubar(ctx) }
            Flow::with_height(TOOLBAR_HEIGHT).expand_across()   => |ctx| { self.toolbar(ctx) }
            Flow::auto(1.0)                                     => |ctx| { self.content(ctx) }
            Flow::with_height(200.0).expand_across()            => |ctx| { self.panel(ctx) }
            Flow::with_height(STATUSBAR_HEIGHT).expand_across() => |ctx| { self.statusbar(ctx) }
        });
        self.second_menubar(ctx);
    }

    fn content(&mut self, ctx: ui::Context<Canvas>) {
        let state = unsafe { &mut *(&self.state as *const UiState as *mut UiState) };

        const WH: usize = 12;
        flow!(h ctx => {
        Flow::with_width(32.0 * 5.0).expand_across() => |ctx| {
            ctx.quad(BAR_BG, ctx.rect());

            let mut lay = EditorLayout::new(ctx, state);

            let mut update_brush = false;
            {
                const WH: f32 = 12.0;
                let dx = self.editor.brush_size.x as usize;
                let dy = self.editor.brush_size.y as usize;
                let size = Vector2::new(WH * dx as f32, WH * dy as f32);
                let dim = Vector2::new(WH, WH);
                let ctx = lay.reserve(WH * 12 as f32, 0.0)
                    .align(Vector2::new(0.5, 0.5), size);
                let start = ctx.rect().min;
                for y in 0..dy {
                    for x in 0..dx {
                        let min = start + Vector2::new(x, y).cast().unwrap() * WH;
                        let r = Rect::from_min_dim(min, dim);

                        let c = &mut self.editor.brush[x + y * dx];
                        if BTN.behavior(&ctx.sub_rect(r), lay.state, &mut ()) {
                            *c = !*c;
                            update_brush = true;
                            self.editor.brush_shape = Shape::Custom;
                        }
                        if *c {
                            ctx.quad(rgba(0xFFFFFF_FF), r.pad(1.0));
                        } else {
                            ctx.quad(rgba(0x000000_FF), r.pad(1.0));
                        }
                    }
                }
            }
            {
                update_brush |= lay.num("size", "x", &mut self.editor.brush_size.x, 1, 1, 12);
                update_brush |= lay.num("size", "y", &mut self.editor.brush_size.y, 1, 1, 12);
                let size = self.editor.brush_size;
                update_brush |= lay.num("offset", "x", &mut self.editor.brush_offset.x, 1, -size.x, size.x);
                update_brush |= lay.num("offset", "y", &mut self.editor.brush_offset.y, 1, -size.y, size.y);
            }

            if update_brush {
                self.editor.resize_brush();
            }


            lay.header_checkbox("Grid", &mut self.grid.visible);
            if self.grid.visible {
                lay.num("size", "x", &mut self.grid.size.x, 1, 0, None);
                lay.num("size", "y", &mut self.grid.size.y, 1, 0, None);
                lay.num("offset", "x", &mut self.grid.offset.x, 1, None, None);
                lay.num("offset", "y", &mut self.grid.offset.y, 1, None, None);
            }
        }
        Flow::auto(1.0) => |ctx| {
            self.rect = ctx.rect().cast().unwrap();
        }
        Flow::with_width(6.0 * WH as f32).expand_across() => |ctx| {
            flow!(v ctx => {
            Flow::with_height(20.0).expand_across() => |ctx| {
                let r = ctx.rect();
                ctx.quad(BAR_TITLE_BG, r);
                //ctx.sub_rect(r.pad_x(8.0)).label(0.0, 0.5, WHITE, "Palette");
                ctx.label(0.5, 0.5, WHITE, "Palette");
            }
            Flow::auto(1.0) => |ctx| {
                ctx.quad(BAR_BG, ctx.rect());
                let r = ctx.rect();
                let start = r.min;

                let transparent = self.editor.transparent();

                let w = (r.dx() as usize) / WH;
                for i in 0..256 {
                    let min = Vector2::new(
                        (i % w) * WH,
                        (i / w) * WH,
                    );
                    let dim = Vector2::new(WH as f32, WH as f32);
                    let r = Rect::from_min_dim(start + min.cast().unwrap(), dim);
                    let color = self.editor.pal(i as u8);

                    if BTN.behavior(&ctx.sub_rect(r), &mut self.state, &mut ()) {
                        self.editor.color = i as u8;
                    }
                    if self.editor.color == i as u8 {
                        BTN.pressed.paint(ctx.draw(), r);
                    }
                    let r = r.pad(0.0);
                    if Some(i as u8) == transparent {
                        let (r1, r2) = r.split_x(0.5);
                        let (a, b) = r1.split_y(0.5);
                        let (c, d) = r2.split_y(0.5);
                        ctx.quad(0x333333_FFu32.to_be(), a);
                        ctx.quad(0x000000_FFu32.to_be(), b);
                        ctx.quad(0x000000_FFu32.to_be(), c);
                        ctx.quad(0x333333_FFu32.to_be(), d);
                    } else {
                        ctx.quad(rgba(color), r);
                    }
                }
            }
            });
        }
        });
    }

    fn panel(&mut self, ctx: ui::Context<Canvas>) {
        ctx.quad(TIMELINE_BG, ctx.rect());

        let state = unsafe { &mut *(&self.state as *const UiState as *mut UiState) };
        let mut lay = EditorLayout::new(ctx, state);
        let m = self.editor.image.as_mut_receiver();
        for layer in &mut m.data {
            let ctx = lay.line();
            let w = ctx.rect().dy();
            flow!(h ctx => {
                Flow::with_width(32.0 * 5.0 - 40.0).expand_across() => |ctx| {
                    ctx.label(0.0, 0.5, WHITE, &layer.name);
                }
                Flow::with_width(w).expand_across() => |ctx| {
                    ::layout::checkbox_inner(ctx, &mut lay.state, &mut layer.lock, (ICON_LOCK, ICON_LOCK_OFF));
                }
                Flow::with_width(w).expand_across() => |ctx| {
                    ::layout::checkbox_inner(ctx, &mut lay.state, &mut layer.visible, (ICON_EYE, ICON_EYE_OFF));
                }
                Flow::auto(1.0) => |ctx| {}
            });
        }
    }

    fn statusbar(&mut self, ctx: ui::Context<Canvas>) {
        ctx.quad(STATUSBAR_BG, ctx.rect());
        let text = format!("zoom: {}  #{:<3}", self.editor.zoom(), self.editor.color);
        ctx.label(0.01, 0.5, WHITE, &text);

        if self.in_widget {
            let text = format!("[{:>+5} {:<+5}]", self.mouse.x, self.mouse.y);
            ctx.label(0.2, 0.5, WHITE, &text);
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.time);
        self.time = now;

        let elapsed = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

        let text = format!("sec: {:.5}", elapsed);
        ctx.label(0.9, 0.5, WHITE, &text);
    }

    fn toolbar(&mut self, ctx: ui::Context<Canvas>) {
        ctx.quad(TOOLBAR_BG, ctx.rect());

        let state = unsafe { &mut *(&self.state as *const UiState as *mut UiState) };

        let height = ctx.rect().dy();
        let btn = Flow::with_wh(height, height);
        let flow = [
            btn, btn, btn, btn, btn,
            Flow::auto(1.0),
            btn, btn,
            Flow::auto(1.0).skip(),
            Flow::with_width(200.0).expand_across(),
            //Flow::auto(1.0),
        ];
        let mut flow = ctx.horizontal_flow(0.0, 0.5, &flow);

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
                    BTN.pressed.paint(ctx.draw(), ctx.rect());
                }
                ctx.draw().texture(icon, r);
            }
        }

        {
            let align = Vector2::new(0.0, 0.5);
            let size = Vector2::new(200.0, 20.0);
            let ctx = flow.next().unwrap().align(align, size);
            let mut lay = EditorLayout::new(ctx, state);
            match self.current {
                CurrentTool::Freehand => {
                    lay.checkbox("perfect", &mut self.freehand.perfect);
                }
                CurrentTool::Primitive(_) => {
                    lay.checkbox("fill", &mut self.prim.fill);
                }
                _ => (),
            }
        }


        {
            let undo = flow.next().unwrap();
            let redo = flow.next().unwrap();

            if BTN.behavior(&redo, &mut self.state, &mut ()) {
                self.editor.redo();
            }
            if BTN.behavior(&undo, &mut self.state, &mut ()) {
                self.editor.undo();
            }

            ctx.draw().texture(ICON_UNDO, undo.rect());
            ctx.draw().texture(ICON_REDO, redo.rect());
        }

        let ctx = flow.next().unwrap();
        match edit_num(ctx, &mut self.state, self.editor.zoom(), "zoom") {
            Some(true) => self.zoom_from_center(1),
            Some(false) => self.zoom_from_center(-1),
            _ => (),
        }
    }

    fn menubar(&mut self, ctx: ui::Context<Canvas>) {
        ctx.quad(MENUBAR_BG, ctx.rect());
        self.file_menu_id = ctx.reserve_widget_id();
        self.brush_menu_id = ctx.reserve_widget_id();
        MENUBAR.run(&ctx, &mut self.state, &mut self.menubar, &[
            (self.file_menu_id, "File"),
            (ctx.reserve_widget_id(), "Edit"),
            (self.brush_menu_id, "Brush"),
            (ctx.reserve_widget_id(), "View"),
            (ctx.reserve_widget_id(), "Tools"),
            (ctx.reserve_widget_id(), "Help"),
        ]);
    }

    fn second_menubar(&mut self, ctx: ui::Context<Canvas>) {
        let mut exit = true;
        match self.menubar.open_root {
        Some((id, base_rect)) if id == self.file_menu_id => {
            match MENU.run(&ctx, &mut self.state, id, base_rect, &FILE_ITEMS) {
                MenuEvent::Nothing => exit = false,
                MenuEvent::Exit => (),
                MenuEvent::Clicked(Command::Open) => {
                    if let Some(name) = ::open::open_file() {
                        println!("open_file: {}", name);
                        if let Some(image) = ::open::load_sprite(name) {
                            self.editor.recreate(image);
                        }
                    }
                }
                MenuEvent::Clicked(Command::Quit) => self.quit = true,
                MenuEvent::Clicked(id) => {
                    println!("click: {:?}", id);
                }
            }
        }
        Some((id, base_rect)) if id == self.brush_menu_id => {
            match MENU_BRUSH.run(&ctx, &mut self.state, id, base_rect, &BRUSH_ITEMS) {
                MenuEvent::Nothing => exit = false,
                MenuEvent::Exit => (),
                MenuEvent::Clicked(cmd) => {
                    self.editor.brush_shape = cmd;
                    self.editor.resize_brush();
                }
            }
        }
        _ => exit = false,
        }
        if exit {
            self.menubar.open_root = None;
        }
    }
}
