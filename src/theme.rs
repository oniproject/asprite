#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use ui::*;
use render::Canvas;
use draw::Shape;

pub const TRANSPARENT: u32 = 0x000000_00;

pub static PAL_GAMEBOY: &[u32] = &[
    0xCADC9F_FF,
    0x0F380F_FF,
    0x306230_FF,
    0x8BAC0F_FF,
    0x9BBC0F_FF,
];

pub const GRID_COLOR: u32 = 0xFF0000_AA;
pub const CORNER_COLOR: u32 = 0x00FF00_AA;


pub const ICON_TOOL_FREEHAND: usize = 1000_0;
pub const ICON_TOOL_FILL: usize = 1000_1;
pub const ICON_TOOL_CIRC: usize = 1000_2;
pub const ICON_TOOL_RECT: usize = 1000_3;
pub const ICON_TOOL_PIP: usize = 1000_4;

pub const ICON_EYE: usize = 1100_0;
pub const ICON_EYE_OFF: usize = 1100_1;

pub const ICON_LOCK: usize = 1100_2;
pub const ICON_LOCK_OFF: usize = 1100_3;

pub const ICON_UNDO: usize = 2000_0;
pub const ICON_REDO: usize = 2000_1;

pub const ICON_CHECK_ON: usize = 2000_2;
pub const ICON_CHECK_OFF: usize = 2000_3;

pub const EDITOR_SPRITE_ID: usize = 4000;

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
*/

pub const TIMELINE_BG: u32 = rgba(0x3A4351_FF);
pub const HEADER_BG: u32 = rgba(0x525b68_FF);

/*
pub const LABEL_COLOR: u32 = 0xFFFFFF_FF;
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

pub const CHECK_ON: TextureDrawer<Canvas> = TextureDrawer(ICON_CHECK_ON);
pub const CHECK_OFF: TextureDrawer<Canvas> = TextureDrawer(ICON_CHECK_OFF);

pub const BTN: ColorButton<Canvas> = ColorButton {
    normal:  ColorDrawer(BTN_NORMAL),
    hovered: ColorDrawer(BTN_HOVERED),
    pressed: ColorDrawer(BTN_PRESSED),
};

pub const TOGGLE_BTN: ColorTransparentButton<Canvas> = ColorTransparentButton {
    normal:  NoDrawer,
    hovered: ColorDrawer(BTN_HOVERED),
    pressed: ColorDrawer(BTN_PRESSED),
};

pub const TOGGLE: ColorTransparentToggle<Canvas> = ColorTransparentToggle {
    checked: TOGGLE_BTN,
    unchecked: TOGGLE_BTN,
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

const MENU_STYLE: MenuStyle<Canvas> = MenuStyle {
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

    width: 200.0,

    text_height: 20.0,
    text_inset: 8.0,
    sep_height: 5.0,
    sep_inset: 2.0,
};

pub const FILE_ITEMS: [Item<Command>; 8] = [
    Item::Text(Command::New, "New", "Ctrl-N"),
    Item::Text(Command::Open, "Open", "Ctrl-O"),
    Item::Text(Command::Recent, "Recent", ">"),
    Item::Separator,
    Item::Text(Command::Save, "Save", "Ctrl-S"),
    Item::Text(Command::SaveAs, "Save as...", "Shift-Ctrl-S"),
    Item::Separator,
    Item::Text(Command::Quit, "Quit", "Ctrl-Q"),
];

pub const BRUSH_ITEMS: [Item<Shape>; 13] = [
    Item::Text(Shape::Round, "Round", ""),
    Item::Text(Shape::Square, "Square", ""),
    Item::Text(Shape::HorizontalBar, "HorizontalBar", ""),
    Item::Text(Shape::VerticalBar, "VerticalBar", ""),
    Item::Text(Shape::Slash, "Slash", ""),
    Item::Text(Shape::Antislash, "Antislash", ""),
    Item::Text(Shape::Cross, "Cross", ""),
    Item::Text(Shape::Plus, "Plus", ""),
    Item::Text(Shape::Diamond, "Diamond", ""),
    Item::Text(Shape::SieveRound, "SieveRound", ""),
    Item::Text(Shape::SieveSquare, "SieveSquare", ""),
    Item::Separator,
    Item::Text(Shape::Custom, "Custom", ""),
];

pub const MENU: Menu<Canvas, Command> = Menu {
    marker: ::std::marker::PhantomData,
    style: MENU_STYLE,
};

pub const MENU_BRUSH: Menu<Canvas, ::draw::Shape> = Menu {
    marker: ::std::marker::PhantomData,
    style: MENU_STYLE,
};
