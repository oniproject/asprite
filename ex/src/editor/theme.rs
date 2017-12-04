#![allow(non_upper_case_globals)]

use math::*;
use ui::*;
use graphics::Graphics;

pub const MENUBAR_BG: [u8; 4] = [0x22, 0x28, 0x33, 0xFF];
pub const MENUBAR_HEIGHT: f32 = 20.0;

pub const TOOLBAR_HEIGHT: f32 = 40.0;
pub const TOOLBAR_BG: [u8; 4] = BAR_BG;

pub const STATUSBAR_BG: [u8; 4] = [0x3F, 0x43, 0x50, 0xFF];
pub const STATUSBAR_HEIGHT: f32 = 20.0;

pub const BAR_BG: [u8; 4] = [0x3F, 0x49, 0x57, 0xFF];
pub const BAR_TITLE_BG: [u8; 4] = [0x52, 0x5b, 0x68, 0xFF];
pub const BAR_TITLE_HEIGHT: f32 = 20.0;

pub const BTN_NORMAL: [u8; 4] = [0x4E, 0x57, 0x63, 0xFF];
pub const BTN_HOVERED: [u8; 4] = [0x3E, 0x48, 0x55, 0xFF];
pub const BTN_PRESSED: [u8; 4] = [0x00, 0x76, 0xFF, 0xFF];

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

pub const GRID_COLOR: u32 = 0xFF0000_FF;
pub const CORNER_COLOR: u32 = 0x00FF00_FF;

pub const TRANSPARENT: u32 = 0x000000_00;
*/

const background: ColorDrawer<Graphics> = ColorDrawer::new([0xFF, 0xFF, 0xFF, 0xCC]);
const fill: ColorDrawer<Graphics> = ColorDrawer::new([0, 0, 0, 0xCC]);

const normal: ColorDrawer<Graphics>  = ColorDrawer::new([0xFF, 0, 0xFF, 0xFF]);
const hovered: ColorDrawer<Graphics> = ColorDrawer::new([0xFF, 0, 0xFF, 0xCC]);
const pressed: ColorDrawer<Graphics> = ColorDrawer::new([0xFF, 0, 0, 0xFF]);

pub const HPROGRESS: Progress<ColorDrawer<Graphics>, ColorDrawer<Graphics>> = Progress { background, fill, axis: Axis::Horizontal };
pub const VPROGRESS: Progress<ColorDrawer<Graphics>, ColorDrawer<Graphics>> = Progress { background, fill, axis: Axis::Vertical };

pub const HSLIDER: Slider<ColorDrawer<Graphics>> = Slider { normal, hovered, pressed, axis: Axis::Horizontal };
pub const VSLIDER: Slider<ColorDrawer<Graphics>> = Slider { normal, hovered, pressed, axis: Axis::Vertical };

pub const BTN: ColorButton<Graphics> = ColorButton {
	normal:  ColorDrawer::new(BTN_NORMAL),
	hovered: ColorDrawer::new(BTN_HOVERED),
	pressed: ColorDrawer::new(BTN_PRESSED),
};

pub const TOGGLE: ColorToggle<Graphics> = Toggle {
	checked: ColorButton {
		normal:   ColorDrawer::new([0xFF, 0, 0, 0xCC]),
		hovered:  ColorDrawer::new([0xFF, 0, 0, 0x99]),
		pressed:  ColorDrawer::new([0xFF, 0, 0, 0x66]),
	},
	unchecked: ColorButton {
		normal:   ColorDrawer::new([0xFF, 0xFF, 0xFF, 0xCC]),
		hovered:  ColorDrawer::new([0xFF, 0xFF, 0xFF, 0x99]),
		pressed:  ColorDrawer::new([0xFF, 0xFF, 0xFF, 0x66]),
	},
};

pub const MENUBAR: menubar::MenuBar<Graphics> = menubar::MenuBar {
	normal_color: [0xFF; 4],
	hover_color:  [0x00, 0x00, 0x00, 0xFF],
	hover_bg:     [0xCC; 4],
};

#[derive(Clone, Debug)]
pub enum Command {
	New, Open, Recent,
	Save, SaveAs,
	Quit,
}

pub const MENU: menubar::Menu<Graphics, Command> = menubar::Menu {
	marker: ::std::marker::PhantomData,
	normal: menubar::ItemStyle {
		label:    [0x00, 0x00, 0x00, 0xFF],
		shortcut: [0x00, 0x00, 0x00, 0x88],
		bg:       [0xFF, 0xFF, 0xFF, 0xFF],
	},
	hovered: menubar::ItemStyle {
		label:    [0x00, 0x00, 0x00, 0xFF],
		shortcut: [0x00, 0x00, 0x00, 0x88],
		bg:       [0xAA, 0xAA, 0xAA, 0xFF],
	},

	separator: [0x00, 0x00, 0x00, 0x99],

	width: 150.0,

	text_height: 20.0,
	text_inset: 8.0,
	sep_height: 5.0,
	sep_inset: 2.0,
};
