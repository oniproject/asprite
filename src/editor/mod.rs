use ui;
use render::Canvas;
use math::*;

mod theme;
mod layout;

use self::theme::*;


pub fn draw_ui(canvas: &mut Canvas, mouse: ui::Mouse) {
    use ui::*;

    /*
    let entity_count = world.entities().join().count();
    let wh = *world.read_resource::<Vector2<f32>>();
    let mouse = *world.read_resource::<Mouse>();
    let graphics = world.read_resource::<Arc<graphics::Graphics>>().clone();
    let dt = world.read_resource::<Time>().delta.seconds;

    let area = &mut self.area;
    */

    let dt = 1.5;
    let entity_count = 500;

    let wh = Vector2::new(::SCREEN_WIDTH as f32, ::SCREEN_HEIGHT as f32);

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
        menubar(&ctx, state);
    }

    if let Some(ctx) = iter.next() {
        let mut add = 0;
        toolbar(&ctx, state, &mut add);
        /*
        for _ in 0..add {
            spawn(world, &self.textures);
        }
        */
    }

    if let Some(ctx) = iter.next() {
        //*area = ctx.rect();

        let hvr = Rect {
            min: Point2::new(0.0, 200.0),
            max: Point2::new(300.0, 400.0),
        };
        let id = ctx.reserve_widget_id();
        ctx.onhover(id, hvr, state,
            || println!("hover start {:?} {:?}", id, hvr),
            || println!("hover end   {:?} {:?}", id, hvr),
        );

        let clk = Rect {
            min: Point2::new(0.0, 400.0),
            max: Point2::new(300.0, 600.0),
        };

        let id = ctx.reserve_widget_id();
        ctx.onclick(id, clk, state, || println!("click {:?} {:?}", id, clk));

        ctx.quad(rgba(0x999999_99), &hvr);
        ctx.quad(rgba(0xAAAAAA_AA), &clk);

        /*
        if false {
            let (_, insp) = ctx.split_x(0.85);
            self.inspector.run(&insp, state, world);
        }
        */
    }

    if let Some(ctx) = iter.next() {
        //xbar(&ctx, state);
    }

    if let Some(ctx) = iter.next() {
        let text = format!("count: {} last: {:?} ms: {:}", entity_count, last_active, dt);
        ctx.label(0.0, 0.5, WHITE, &text);
    }

    if true {
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


    second_menubar(&ctx, state);
}




static mut MENUBAR_MODEL: ui::menubar::MenuBarModel = ui::menubar::MenuBarModel {
	open_root: None,
};

fn menubar(ctx: &ui::Context<Canvas>, state: &mut ui::UiState) {
	MENUBAR.run(&ctx, state, unsafe { &mut MENUBAR_MODEL }, &[
		(ctx.reserve_widget_id(), "File"),
		(ctx.reserve_widget_id(), "Edit"),
		(ctx.reserve_widget_id(), "View"),
		(ctx.reserve_widget_id(), "Tools"),
		(ctx.reserve_widget_id(), "Help"),
	]);
}

fn toolbar(ctx: &ui::Context<Canvas>, state: &mut ui::UiState, add: &mut usize) {
	use ui::*;
	let widgets = &[
		Flow::with_wh(80.0, 40.0),
		Flow::with_wh(80.0, 40.0),
		Flow::with_wh(80.0, 40.0),
		Flow::with_wh(80.0, 40.0),
	];
	let mut to_add = 1;
	for ctx in ctx.horizontal_flow(0.2, 0.0, widgets) {
		if BTN.behavior(&ctx, state, &mut ()) {
			*add = to_add;
		}
		ctx.label(0.5, 0.5, WHITE, &format!("add {}", to_add));
		to_add *= 10;
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

fn second_menubar(ctx: &ui::Context<Canvas>, state: &mut ui::UiState) {
	use ui::menubar::*;
	use ui::menubar::MenuEvent::*;
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

	let menubar = unsafe { &mut MENUBAR_MODEL };
	if let Some((id, base_rect)) = menubar.open_root {
		let exit = match MENU.run(&ctx, state, id, base_rect, &items) {
			Nothing => false,
			Exit => true,
			Clicked(id) => {
				println!("click: {:?}", id);
				true
			}
		};
		if exit {
			unsafe { MENUBAR_MODEL.open_root = None; }
		}
	}
}
