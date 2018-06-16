#![feature(pattern_parentheses)]
#![feature(const_fn)]
#![feature(associated_type_defaults)]
#![feature(step_trait)]
#![feature(generators, generator_trait)]
#![feature(const_cell_new)]

extern crate rand;
extern crate undo;
extern crate image;
extern crate nfd;
extern crate cgmath;
#[macro_use]
extern crate derivative;

extern crate sdl2;
extern crate specs;
#[macro_use]
extern crate specs_derive;

use sdl2::ttf::{self, Sdl2TtfContext};
use specs::prelude::*;

#[macro_export]
macro_rules! rect(
	($x:expr, $y:expr, $w:expr, $h:expr) => (
		$crate::sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
	)
);

#[macro_export]
macro_rules! color(
	($c:expr) => (
		{
			use $crate::sdl2::gfx::primitives::ToColor;
			let (r, g, b, a) = $c.to_be().as_rgba();
			$crate::sdl2::pixels::Color::RGBA(r, g, b, a)
		}
	)
);


mod math;
mod ui;
mod common;
mod cmd;
mod tool;

mod app;
mod theme;
mod grid;
mod ed;

mod util;
mod button;
mod line;
mod render;
mod widget;

mod editor;

use line::*;
use button::*;

const SCREEN_TITLE: &str = "rust-sdl2_gfx: draw line & FPSManager";
pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 720;
const FONT: &str = "./assets/font/Roboto-Bold.ttf";

fn main() {
    //let ttf = sdl2::ttf::init().unwrap();
    //let font = ttf.load_font(FONT, 12).unwrap();

    let r = render::Canvas::new(SCREEN_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT, FONT);

    let mut world = World::new();
    world.add_bundle(render::Bundle);

    world.create_entity().with(Button::new(30, 50, "ok", || println!("click OK"))).build();
    world.create_entity().with(Button::new(80, 50, "cancel", || println!("click CANCEL"))).build();

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(r)
        .build();


    let sprite = {
        use math::*;
        use common::gradient;
        use common::Palette;

        fn create_pal(pal: &mut Palette<u32>) {
            const GB0: u32 = 0xCADC9F_FF;
            const GB1: u32 = 0x0F380F_FF;
            const GB2: u32 = 0x306230_FF;
            const GB3: u32 = 0x8BAC0F_FF;
            const GB4: u32 = 0x9BBC0F_FF;

            pal[0] = GB0;
            pal[1] = GB1;
            pal[2] = GB2;
            pal[3] = GB3;
            pal[4] = GB4;
        }

        let mut sprite = cmd::Sprite::new("GEN", 160, 120);
        sprite.add_layer("Layer Down");
        sprite.add_layer("Layer 2");
        sprite.add_layer("Layer 3");
        sprite.add_layer("Layer 4");
        sprite.add_layer("Layer Up");

        create_pal(&mut sprite.palette);

        if true {
            let page = sprite.page_mut(0, 0);

            let r = Rect::from_coords_and_size(0i32, 0, 160, 120);
            let va = Point2::new(20i32, 10);
            let vb = Point2::new(130i32, 100);

            gradient::draw_gradient(r, va, vb, |p, idx, total| {
                let pos = gradient::extra_dithered(idx, p.x as i16, p.y as i16, total, 5, 1);
                let ii = p.x + p.y * page.width as i32;
                page.page[ii as usize] = pos as u8;
            });
        }
        sprite
    };

    dispatcher.setup(&mut world.res);

    while !*world.read_resource::<bool>() {
        dispatcher.dispatch(&mut world.res);
    }


}
