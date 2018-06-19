#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use math::*;
use ui::*;
use render::Canvas;


pub const TRANSPARENT: u32 = 0x000000_00;

pub static PAL_GAMEBOY: &[u32] = &[
    0xCADC9F_FF,
    0x0F380F_FF,
    0x306230_FF,
    0x8BAC0F_FF,
    0x9BBC0F_FF,
];

pub const EDITOR_SPRITE_ID: usize = 777;
pub const EDITOR_PREVIEW_ID: usize = 888;

pub const GRID_COLOR: u32 = 0xFF0000_AA;
pub const CORNER_COLOR: u32 = 0x00FF00_AA;


pub const ICON_TOOL_FREEHAND: usize = 1000_0;
pub const ICON_TOOL_FILL: usize = 1000_1;
pub const ICON_TOOL_CIRC: usize = 1000_2;
pub const ICON_TOOL_RECT: usize = 1000_3;
pub const ICON_TOOL_PIP: usize = 1000_4;

pub const ICON_UNDO: usize = 2000_0;
pub const ICON_REDO: usize = 2000_1;

pub const fn rgba(c: u32) -> u32 {
    ((c >> 24) & 0xFF) <<  0 |
    ((c >> 16) & 0xFF) <<  8 |
    ((c >>  8) & 0xFF) << 16 |
    ((c >>  0) & 0xFF) << 24
}

pub const WHITE: u32 = rgba(0xFFFFFF_FF);

pub const MENUBAR_BG: u32 = rgba(0x222833_FF);
pub const MENUBAR_HEIGHT: f32 = 20.0;

pub const TOOLBAR_HEIGHT: f32 = 32.0;
pub const TOOLBAR_BG: u32 = BAR_BG;

pub const STATUSBAR_BG: u32 = rgba(0x3F4350_FF);
pub const STATUSBAR_HEIGHT: f32 = 20.0;

pub const BAR_BG: u32 = rgba(0x3F4957_FF);
pub const BAR_TITLE_BG: u32 = rgba(0x525b68_FF);
pub const BAR_TITLE_HEIGHT: f32 = 20.0;

pub const BTN_NORMAL: u32 =  rgba(0x4E5763_FF);
pub const BTN_HOVERED: u32 = rgba(0x3E4855_FF);
pub const BTN_PRESSED: u32 = rgba(0x0076FF_FF);

/*
pub const WINDOW_BG: u32 = 0x20242F_FF;

pub const STATUSBAR_BG: u32 = 0x3F4350_FF;
pub const STATUSBAR_COLOR: u32 = 0xA7A8AE_FF;
pub const MENUBAR_BG: u32 = 0x222833_FF;
pub const BAR_BG: u32 = 0x3f4957_FF;

pub const BTN_BORDER: u32 = ;

pub const TIMELINE_BG: u32 = 0x3a4351_FF;
pub const HEADER_BG: u32 = 0x525b68_FF;

pub const LABEL_COLOR: u32 = 0xFFFFFF_FF;
pub const FONT_HEIGHT: u16 = 12;

pub const INSET_X: i16 = FONT_HEIGHT as i16 / 2;
*/

const background: ColorDrawer<Canvas> = ColorDrawer(rgba(0xFFFFFF_CC));
const fill: ColorDrawer<Canvas> = ColorDrawer(rgba(0x000000_CC));

const normal: ColorDrawer<Canvas>  = ColorDrawer(rgba(0xFF00FF_FF));
const hovered: ColorDrawer<Canvas> = ColorDrawer(rgba(0xFF00FF_CC));
const pressed: ColorDrawer<Canvas> = ColorDrawer(rgba(0xFF0000_FF));

pub const HPROGRESS: Progress<ColorDrawer<Canvas>, ColorDrawer<Canvas>> = Progress { background, fill, axis: Axis::Horizontal };
pub const VPROGRESS: Progress<ColorDrawer<Canvas>, ColorDrawer<Canvas>> = Progress { background, fill, axis: Axis::Vertical };

pub const HSLIDER: Slider<ColorDrawer<Canvas>> = Slider { normal, hovered, pressed, axis: Axis::Horizontal };
pub const VSLIDER: Slider<ColorDrawer<Canvas>> = Slider { normal, hovered, pressed, axis: Axis::Vertical };

pub const BTN: ColorButton<Canvas> = ColorButton {
	normal:  ColorDrawer(BTN_NORMAL),
	hovered: ColorDrawer(BTN_HOVERED),
	pressed: ColorDrawer(BTN_PRESSED),
};

pub const TOGGLE: ColorToggle<Canvas> = Toggle {
	checked: ColorButton {
		normal:   ColorDrawer(rgba(0xFF0000_CC)),
		hovered:  ColorDrawer(rgba(0xFF0000_99)),
		pressed:  ColorDrawer(rgba(0xFF0000_66)),
	},
	unchecked: ColorButton {
		normal:   ColorDrawer(rgba(0xFFFFFF_CC)),
		hovered:  ColorDrawer(rgba(0xFFFFFF_99)),
		pressed:  ColorDrawer(rgba(0xFFFFFF_66)),
	},
};

pub const MENUBAR: MenuBar<Canvas> = MenuBar {
	normal_color: rgba(0xFFFFFF_FF),
	hover_color:  rgba(0x000000_FF),
	hover_bg:     rgba(0xCCCCCC_CC),
};

#[derive(Clone, Debug)]
pub enum Command {
	New, Open, Recent,
	Save, SaveAs,
	Quit,
}

pub const MENU: Menu<Canvas, Command> = Menu {
	marker: ::std::marker::PhantomData,
	normal: ItemStyle {
		label:    rgba(0x000000_FF),
		shortcut: rgba(0x000000_88),
		bg:       rgba(0xFFFFFF_FF),
	},
	hovered: ItemStyle {
		label:    rgba(0x000000_FF),
		shortcut: rgba(0x000000_88),
		bg:       rgba(0xAAAAAA_FF),
	},

	separator: rgba(0x000000_99),

	width: 150.0,

	text_height: 20.0,
	text_inset: 8.0,
	sep_height: 5.0,
	sep_inset: 2.0,
};
