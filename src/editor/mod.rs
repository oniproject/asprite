use render::Canvas;
use math::*;

pub mod theme;
mod layout;

use self::theme::*;

use ui;
use ui::*;

use ed::CurrentTool;
use tool::PrimitiveMode;
use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use math::*;

use ed::*;
use cmd::*;
use ui::*;
use render;

use draw::Sprite;
use cmd::{ImageCell, image_cell};
use ed::Tools;

pub struct App {
    pub init: bool,
    pub update: bool,
    pub quit: bool,

    pub files: Vec<ImageCell>,
    pub current: usize,

    pub tools: Tools,

    pub menubar: MenuBarModel,
}

impl App {
    pub fn new(sprite: Sprite) -> Self {
        use tool::Context;
        let files = vec![image_cell(sprite)];

        let mut tools = Tools::new(1, Point2::new(300, 200), files[0].clone());
        tools.editor.sync();
        Self {
            init: false,

            update: true,
            quit: false,
            current: 0,
            files,
            tools,
            menubar: MenuBarModel { open_root: None },
        }
    }

    pub fn event(&mut self, event: sdl2::event::Event) {
        use tool::*;

        self.update = true;
        match event {
        Event::MouseMotion {x, y, xrel, yrel, ..} => {
            //let p = Point2::new(x as i16, y as i16);
            //render.mouse(Mouse::Move(p));
            let p = Point2::new(x as i32, y as i32);
            let v = Vector2::new(xrel as i32, yrel as i32);
            self.tools.mouse_move(p, v);
        }

        Event::Quit {..} => self.quit = true,

        Event::KeyUp { keycode: Some(keycode), .. } => {
            match keycode {
                Keycode::LShift |
                Keycode::RShift =>
                    self.tools.input(Input::Special(false)),
                Keycode::LCtrl |
                Keycode::RCtrl =>
                    self.tools.drag = false,
                _ => (),
            }
        }

        Event::KeyDown { keycode: Some(keycode), keymod, ..} => {
            let shift = keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD);
            let _alt = keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD);
            let _ctrl = keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD);
            match keycode {
                Keycode::Escape => self.tools.input(Input::Cancel),

                Keycode::Plus  | Keycode::KpPlus  => self.tools.zoom_from_center(1),
                Keycode::Minus | Keycode::KpMinus => self.tools.zoom_from_center(-1),

                Keycode::LShift |
                Keycode::RShift =>
                    self.tools.input(Input::Special(true)),

                Keycode::LCtrl |
                Keycode::RCtrl =>
                    self.tools.drag = true,

                Keycode::U => self.tools.undo(),
                Keycode::R => self.tools.redo(),

                //Keycode::Tab if shift => render.key = Some(gui::Key::PrevWidget),
                //Keycode::Tab if !shift => render.key = Some(gui::Key::NextWidget),

                _ => (),
            }
        }

        Event::MouseButtonDown { mouse_btn: MouseButton::Middle, .. } => {
            self.tools.drag = true;
        }
        Event::MouseButtonUp { mouse_btn: MouseButton::Middle, .. } => {
            self.tools.drag = false;
        }

        Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
            //let p = Point2::new(x as i16, y as i16);
            //render.mouse(Mouse::Press(p));
            let p = Point2::new(x as i32, y as i32);
            self.tools.mouse_press(p);
        }
        Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
            //let p = Point2::new(x as i16, y as i16);
            //render.mouse(Mouse::Release(p));
            let p = Point2::new(x as i32, y as i32);
            self.tools.mouse_release(p);
        }

        Event::MouseWheel { y, ..} => { self.tools.zoom_from_mouse(y as i32); }

        _ => (),
        }
    }

    pub fn draw_ui(&mut self, canvas: &mut Canvas, mouse: ui::Mouse) {
        if !self.init {
            use tool::Context;
            self.init = true;
            self.tools.editor.change_color(2);
            canvas.load_texture(ICON_TOOL_FREEHAND, "res/tool_freehand.png");
            canvas.load_texture(ICON_TOOL_PIP, "res/tool_pip.png");
            canvas.load_texture(ICON_TOOL_RECT, "res/tool_rect.png");
            canvas.load_texture(ICON_TOOL_CIRC, "res/tool_circ.png");
            canvas.load_texture(ICON_TOOL_FILL, "res/tool_fill.png");

            canvas.load_texture(ICON_UNDO, "res/undo.png");
            canvas.load_texture(ICON_REDO, "res/redo.png");
        }

        {
            let canvas = canvas.canvas.get_mut();
            canvas.set_draw_color(::sdl2::pixels::Color::RGB(0xFF, 0xFF, 0xFF));
            canvas.clear();
        }

        self.tools.paint(canvas);

        let dt = 1.5;
        let entity_count = 500;

        let (w, h) = canvas.canvas.borrow().logical_size();
        let wh = Vector2::new(w as f32, h as f32);

        let rect = Rect::from_min_dim(Point2::new(0.0, 0.0), wh);
        let ctx = Context::new(canvas, rect, mouse);

        static mut STATE: UiState = UiState::new();
        let state = unsafe { &mut STATE };
        let last_active = state.active_widget();



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

        if let Some(ctx) = iter.next() {
            self.menubar(&ctx, state);
        }

        if let Some(ctx) = iter.next() {
            let mut add = 0;
            self.toolbar(&ctx, state, &mut add);
            /*
            for _ in 0..add {
                spawn(world, &self.textures);
            }
            */
        }

        if let Some(ctx) = iter.next() {
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
                    let color = self.tools.pal(i as u8);

                    if BTN.behavior(&ctx.sub_rect(r), state, &mut ()) {
                        self.tools.editor.change_color(i as u8);
                    }
                    ctx.quad(rgba(color), &r.pad(1.0));
                }
            }
        }

        if let Some(_ctx) = iter.next() {
            //xbar(&ctx, state);
        }

        if let Some(ctx) = iter.next() {

            let text = format!("zoom: {}  #{:<3}", self.tools.zoom, self.tools.color_index());
            ctx.label(0.01, 0.5, WHITE, &text);
            let text = format!("[{:>+5} {:<+5}]", self.tools.mouse.x, self.tools.mouse.y);
            ctx.label(0.2, 0.5, WHITE, &text);
            let text = format!("count: {} last: {:?} ms: {:}", entity_count, last_active, dt);
            ctx.label(0.5, 0.5, WHITE, &text);
        }


        if false {
            self.sample(&ctx, state);
        }

        self.second_menubar(&ctx, state);
    }

    fn sample(&mut self, ctx: &ui::Context<Canvas>, state: &mut ui::UiState) {
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
                TOGGLE.behavior(&ctx, state, toggle_state);
                ctx.label(0.5, 0.5, WHITE, &format!("tgl{}", i));
            }
            2 => {
                HSLIDER.behavior(&ctx, state, slider_state_h);
                ctx.label(0.5, 0.5, WHITE, &format!("val{}: {}", i, slider_state_h.current));
            }
            4 => {
                VSLIDER.behavior(&ctx, state, slider_state_v);
                ctx.label(0.5, 0.5, WHITE, &format!("val{}: {}", i, slider_state_v.current));
            }
            _ => {
                if BTN.behavior(&ctx, state, &mut ()) {
                    println!("{} click", i);
                }
                ctx.label(0.5, 0.5, WHITE, &format!("btn{}", i));
            }
            }

            i += 1;
        }
    }

    fn toolbar(&mut self, ctx: &ui::Context<Canvas>, state: &mut ui::UiState, add: &mut usize) {
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

            if BTN.behavior(&redo, state, &mut ()) {
                self.tools.redo();
            }
            if BTN.behavior(&undo, state, &mut ()) {
                self.tools.undo();
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
                if BTN.behavior(&ctx, state, &mut ()) {
                    self.tools.current = tool;
                }
                let r = ctx.rect();
                if self.tools.current == tool {
                    BTN.pressed.draw_frame(ctx.draw(), ctx.rect());
                }
                ctx.draw().texture(&icon, &r);
            }
        }

        let _ = flow.next().unwrap();
        let ctx = flow.next().unwrap();

        match layout::edit_num(&ctx, state, self.tools.zoom, "zoom") {
            Some(true) => self.tools.zoom_from_center(1),
            Some(false) => self.tools.zoom_from_center(-1),
            _ => (),
        }
    }

    /*
    fn xbar(ctx: &ui::Context<graphics::Graphics>, _state: &mut ui::UiState) {
        use ui::*;
        use math::*;
        use graphics::CustomCmd;

        let pos = ctx.rect().min;

        let mut proj = Affine::one();
        //proj.scale(5.0, 5.0);
        proj.scale(50.0, 50.0);
        proj.translate(pos.x + 150.0, pos.y + 30.0);

        ctx.custom(CustomCmd::Fill(rgba(0xFFFFFF_AA), proj, {
            use lyon::math::*;
            use lyon::path::*;
            use lyon::path::builder::*;

            let mut builder = SvgPathBuilder::new(Path::builder());
            if false {
                //lyon::extra::rust_logo::build_logo_path(&mut builder);
            } else {
                builder.move_to(point(0.0, 0.0));
                builder.line_to(point(1.0, 0.0));
                builder.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                builder.cubic_bezier_to(point(1.0, 1.0), point(0.0, 1.0), point(0.0, 0.0));
                builder.close();
            }
            builder.build()
        }));
    }
    */

    fn menubar(&mut self, ctx: &ui::Context<Canvas>, state: &mut ui::UiState) {
        MENUBAR.run(&ctx, state, &mut self.menubar, &[
            (ctx.reserve_widget_id(), "File"),
            (ctx.reserve_widget_id(), "Edit"),
            (ctx.reserve_widget_id(), "View"),
            (ctx.reserve_widget_id(), "Tools"),
            (ctx.reserve_widget_id(), "Help"),
        ]);
    }

    fn second_menubar(&mut self, ctx: &ui::Context<Canvas>, state: &mut ui::UiState) {
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
            let exit = match MENU.run(&ctx, state, id, base_rect, &items) {
                MenuEvent::Nothing => false,
                MenuEvent::Exit => true,
                MenuEvent::Clicked(id) => {
                    println!("click: {:?}", id);
                    true
                }
            };
            if exit {
                self.menubar.open_root = None;
            }
        }
    }
}
