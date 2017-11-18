use super::*;

use std::cell::{Cell, RefCell};

pub type Color = u32;

const BLACK: Color = 0x000000_FF;
const SELECT_BLUE: Color = 0x5294E2_FF;
const BORDER_GREY: Color = 0xCFD6E6_FF;
const WINDOW_GREY: Color = 0xF5F6F7_FF;
const BUTTON_WHITE: Color = 0xFBFBFC_FF;
const WHITE: Color = 0xFFFFFF_FF;

pub static WINDOW_BACKGROUND: Color = WINDOW_GREY;

pub static LABEL_COLOR: Color = BLACK;

pub static BUTTON_BACKGROUND: Color = BUTTON_WHITE;
pub static BUTTON_BG_SELECTION: Color = SELECT_BLUE;
pub static BUTTON_BORDER: Color = BORDER_GREY;
pub static BUTTON_FOREGROUND: Color = BLACK;
pub static BUTTON_FG_SELECTION: Color = WHITE;

pub static ITEM_BACKGROUND: Color = WHITE;
pub static ITEM_BORDER: Color = BORDER_GREY;
pub static ITEM_FOREGROUND: Color = BLACK;
pub static ITEM_SELECTION: Color = SELECT_BLUE;

pub static TEXT_BACKGROUND: Color = WHITE;
pub static TEXT_BORDER: Color = BORDER_GREY;
pub static TEXT_FOREGROUND: Color = BLACK;
pub static TEXT_SELECTION: Color = SELECT_BLUE;

impl<N: BaseNum> Root<N, u32> {
	pub fn new(rect: Rect<N>) -> Self {
		Self {
			widgets: RefCell::new(Vec::new()),
			focus: Cell::new(usize::max_value()),
			rect: Cell::new(rect),
			redraw: Cell::new(true),
			mouse_pos: Cell::new(Point2::new(N::zero(), N::zero())),
			mouse_left: Cell::new(false),
			bg: Cell::new(WINDOW_BACKGROUND),
		}
	}
}

impl<N: BaseNum> Button<N, u32> {
	pub fn new<F: Fn(&Self) + 'static>(label: String, callback: F) -> Rc<Self> {
		Rc::new(Self {
			bg: (BUTTON_BACKGROUND, BUTTON_BG_SELECTION),
			fg: (BUTTON_FOREGROUND, BUTTON_FG_SELECTION),
			border: Some((BUTTON_BORDER, BUTTON_BORDER)),
			label: RefCell::new(label),
			callback: RefCell::new(Rc::new(callback)),
			pressed: Cell::new(false),

			rect: Cell::new(Rect::default()),
			measured: Cell::new(Vector2::zero()),
			layout: RefCell::new(FlowData {
				along_weight: N::zero(),
				expand_along: false,
				shrink_along: false,
				expand_across: true,
				shrink_across: true,
			}),
		})
	}
}

impl<N: BaseNum> Label<N, u32> {
	pub fn new(label: String) -> Rc<Self> {
		Rc::new(Self {
			rect: Cell::new(Rect::default()),
			measured: Cell::new(Vector2::zero()),

			color: LABEL_COLOR,
			label: RefCell::new(label),
		})
	}
}