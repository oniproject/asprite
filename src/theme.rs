/*
editor
	bg - 20242F 

toolbar
	37x37
	icon - ECECEC
	bg - 353D4B
	active - 0076FF
	header - 6e757f



shift - lines
ctrl  - hold by 30deg (momevent)
alt   - picker

normal:
	left - paint
	middle - move / zoom
	right - picker

shift:
	left - line paint
	middle - ???
	right - ???

ctlr:

alt:

*/

pub const WINDOW_BG: u32 = 0x20242F_FF;

pub const STATUSBAR_BG: u32 = 0x3F4350_FF;
pub const STATUSBAR_COLOR: u32 = 0xA7A8AE_FF;
pub const MENUBAR_BG: u32 = 0x222833_FF;
pub const BAR_BG: u32 = 0x3f4957_FF;

pub const BTN_BORDER: u32 = 0x4E5763_FF;
pub const BTN_BG: u32 = 0x3E4855_FF;
pub const BTN_ACTIVE: u32 = 0x0076FF_FF;

pub const TIMELINE_BG: u32 = 0x3a4351_FF;
pub const HEADER_BG: u32 = 0x525b68_FF;

pub const LABEL_COLOR: u32 = 0xFFFFFF_FF;
pub const FONT_HEIGHT: u16 = 12;

pub const INSET_X: i16 = FONT_HEIGHT as i16 / 2;

pub const ICON_TOOL_FREEHAND: usize = 1000_0;
pub const ICON_TOOL_FILL: usize = 1000_1;
pub const ICON_TOOL_CIRC: usize = 1000_2;
pub const ICON_TOOL_RECT: usize = 1000_3;
pub const ICON_TOOL_PIP: usize = 1000_4;

pub const EDITOR_SPRITE_ID: usize = 777;
pub const EDITOR_PREVIEW_ID: usize = 888;

pub const GRID_COLOR: u32 = 0xFF0000_FF;
pub const CORNER_COLOR: u32 = 0x00FF00_FF;

pub const TRANSPARENT: u32 = 0x000000_00;
