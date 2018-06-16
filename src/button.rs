use sdl2::render::WindowCanvas;
use sdl2::gfx::primitives::DrawRenderer;

use specs::prelude::*;
use util::*;

use ui::Graphics;
use math::*;

use std::sync::Arc;

const fn rgba(c: u32) -> u32 {
    ((c >> 24) & 0xFF) <<  0 |
    ((c >> 16) & 0xFF) <<  8 |
    ((c >>  8) & 0xFF) << 16 |
    ((c >>  0) & 0xFF) << 24
}

/*
const TEXT: u32 =   rgba(0xFFFFFF_FF);
const NORMAL: u32 = rgba(0x666666_FF);
const HOVER: u32 =  rgba(0x6666CC_FF);
const ACTIVE: u32 = rgba(0xCC0000_FF);
*/

const TEXT: u32 =   rgba(0x000000_FF);
const NORMAL: u32 = rgba(0xFFFFFF_00);
const HOVER: u32 =  rgba(0x000000_0A);
//const ACTIVE: u32 = rgba(0x000000_28);
const ACTIVE: u32 = rgba(0x000000_33);

#[derive(Component)]
#[storage(VecStorage)]
pub struct Button {
    x: i16,
    y: i16,
    pad: i16,
    label: String,
    bg: u32,
    color: u32,
    active: bool,
    callback: Arc<Fn() + Sync + Send>,
}

impl Button {
    pub fn new<F>(x: i16, y: i16, label: &str, callback: F) -> Self
        where F: Fn() + Sync + Send + 'static
    {
        Self {
            x, y,
            callback: Arc::new(callback),
            //label: label.to_uppercase(),
            label: label.into(),
            pad: 8,
            bg: NORMAL,
            color: TEXT,
            active: false,
        }
    }

    fn check(&self, pos: (i16, i16)) -> bool {
        let (w, h) = measure_text(&self.label);
        pos.0 >= (self.x - self.pad) &&
        pos.1 >= (self.y - self.pad) &&
        pos.0 <= (self.x + self.pad + w) &&
        pos.1 <= (self.y + self.pad + h)
    }

    pub fn mouse(&mut self, pos: (i16, i16), down: Option<bool>) {
        let hover = self.check(pos);

        if let Some(active) = down {
            if self.active && hover && !active {
                (*self.callback)();
            }
            self.active = active;
        }

        if self.active && hover {
            self.bg = ACTIVE;
        } else {
            self.bg = if hover { HOVER } else { NORMAL };
        }
    }

    pub fn paint(&self, canvas: &mut ::render::Canvas) {
        let size = canvas.measure_text(&self.label);

        canvas.quad(self.bg, &Rect::from_coords(
            (self.x - self.pad) as f32,
            (self.y - self.pad) as f32,
            (self.x + self.pad + size.x as i16) as f32,
            (self.y + self.pad + size.y as i16) as f32,
        ));

        let p = Point2::new(self.x as f32, self.y as f32);
        canvas.text(p, self.color, &self.label);

        /*
        let _ = canvas.hline(sx+0, ex-0, sy, rgba(0x000000_33));
        let _ = canvas.hline(sx+0, ex-0, ey, rgba(0x000000_33));
        let _ = canvas.vline(sx, sy+1, ey-1, rgba(0x000000_33));
        let _ = canvas.vline(ex, sy+1, ey-1, rgba(0x000000_33));
        */
    }
}

/*
enum ButtonColor {
    Normal,
    Accent,
    Primary,
}

enum ButtonEmphasis {
    Low,    // text
    Medium, // outlined
    High,   // contained
}

enum ButtonState {
    Enabled,
    Disabled,
    Hovered,
    Focused,
    Pressed,
}
*/
