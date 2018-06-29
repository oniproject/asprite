#![feature(pattern_parentheses)]
#![feature(const_fn)]
#![feature(associated_type_defaults)]
#![feature(step_trait)]
#![feature(generators, generator_trait)]
#![feature(const_cell_new)]
#![feature(decl_macro)]

extern crate rand;
extern crate redo;
extern crate image;
extern crate nfd;
extern crate cgmath;
#[macro_use]
extern crate derivative;

extern crate sdl2;
extern crate specs;
#[macro_use]
extern crate specs_derive;

use specs::prelude::*;

/*
#[macro_export]
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        $crate::sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);
*/

pub macro rect($x:expr, $y:expr, $w:expr, $h:expr) {
    $crate::sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
}

pub macro color($c:expr) {
    use $crate::sdl2::gfx::primitives::ToColor;
    let (r, g, b, a) = $c.to_be().as_rgba();
    $crate::sdl2::pixels::Color::RGBA(r, g, b, a)
}

mod math;
mod draw;
mod ui;
mod tool;

//mod editor;
//
mod app;
mod grid;
mod theme;
mod layout;

mod prev;

mod open;
mod render;

const SCREEN_TITLE: &str = "rust-sdl2_gfx: draw line & FPSManager";
pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 720;

fn main() {
    let r = render::Canvas::new(SCREEN_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut world = World::new();
    world.add_bundle(render::Bundle);
    world.add_resource(app::App::new(new_sprite()));

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(r)
        .build();

    dispatcher.setup(&mut world.res);

    while !*world.read_resource::<bool>() {
        dispatcher.dispatch(&mut world.res);
    }
}

fn new_sprite() -> tool::Receiver {
    use math::*;
    use draw::*;

    const _TRANSPARENT: u32 = 0x000000_00;

    const BLACK: u32    = 0x18140C_FF;
    const WHITE: u32    = 0xF4F0E8_FF;
    const RED: u32      = 0xC44448_FF;
    const GREEN: u32    = 0x30845C_FF;
    const YELLOW: u32   = 0xF0E848_FF;
    const VIOLET: u32   = 0x343074_FF;
    const PINK: u32     = 0xBC306C_FF;
    const BLUE: u32     = 0x2874C4_FF;

    static PAL: &[u32] = &[
        BLACK,
        WHITE,
        RED,
        GREEN,
        YELLOW,
        VIOLET,
        PINK,
        BLUE,
    ];

    static WARM: &[u32] = &[
        0xC40909_FF,
        0xD8D508_FF,
        0xED6E00_FF,
        0xE80032_FF,
        0x8C0B0B_FF,
        0xE4AA04_FF,
        0x750000_FF,
    ];

    static COOL: &[u32] = &[
        0x112AC6_FF,
        0x539BE2_FF,
        0x161066_FF,
        0x40234C_FF,
        0x073F93_FF,
        0x2C6CCC_FF,
        0x265121_FF,
        0x04422C_FF,
    ];

    fn create_pal(pal: &mut Palette<u32>) {
        for c in pal.iter_mut() {
            *c = 0x000000_FF;
        }
        for (i, &c) in WARM.iter().enumerate() {
            pal[i as u8 + 1] = c;
        }
        for (i, &c) in COOL.iter().enumerate() {
            pal[i as u8 + 8] = c;
        }

        for (i, &c) in PAL.iter().enumerate() {
            pal[i as u8 + 24] = c;
        }

        let arr = [0xFFu32, 0xCC, 0x99, 0x66, 0x33, 0x00];
        let mut idx = 36u8;
        for b in &arr[..] {
            for g in &arr[..] {
                for r in &arr[..] {
                    pal[idx] = (r << 24) | (g << 16) | (b << 8) | 0xFFu32;
                    idx += 1;
                }
            }
        }
    }

    let mut sprite = tool::Receiver::new("GEN", 160, 120);
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
}
