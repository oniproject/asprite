#![allow(dead_code)]

use std::mem;

type Layers = [TileData; MAX_LAYERS];
type Props = [f32; MAX_PROPERTIES];

fn ALLOW_LINK(_tile_layers: &Layers, _tile_props: &Props, _link_layers: &Layers, _link_props: &Props) -> bool {
	true
}







/*
#ifdef STB_TILEMAP_EDITOR_IMPLEMENTATION

#ifndef STBTE_ASSERT
#define STBTE_ASSERT assert
#include <assert.h>
#endif

#ifdef _MSC_VER
#define STBTE__NOTUSED(v)  (void)(v)
#else
#define STBTE__NOTUSED(v)  (void)sizeof(v)
#endif
*/

const MAX_TILEMAP_X: usize = 200;
const MAX_TILEMAP_Y: usize = 200;
const MAX_LAYERS: usize = 8;
const MAX_CATEGORIES: usize = 100;

/*
#ifndef STBTE_MAX_COPY
#define STBTE_MAX_COPY			  65536
#endif
*/

const UNDO_BUFFER_BYTES: usize = 1 << 24; // 16 MB

/*
#ifndef STBTE_PROP_TYPE
#define STBTE__NO_PROPS
#define STBTE_PROP_TYPE(n,td,tp)	0
#endif

#ifndef STBTE_PROP_NAME
#define STBTE_PROP_NAME(n,td,tp)  ""
#endif
*/

const MAX_PROPERTIES: usize = 10;

/*
#ifndef STBTE_PROP_MIN
#define STBTE_PROP_MIN(n,td,tp)  0
#endif

#ifndef STBTE_PROP_MAX
#define STBTE_PROP_MAX(n,td,tp)  100.0
#endif

#ifndef STBTE_PROP_FLOAT_SCALE
#define STBTE_PROP_FLOAT_SCALE(n,td,tp)  1	// default scale size
#endif

#ifndef STBTE_FLOAT_CONTROL_GRANULARITY
#define STBTE_FLOAT_CONTROL_GRANULARITY 4
#endif


#define STBTE__UNDO_BUFFER_COUNT  (STBTE_UNDO_BUFFER_BYTES>>1)

#if STBTE_MAX_TILEMAP_X > 4096 || STBTE_MAX_TILEMAP_Y > 4096
#error "Maximum editable map size is 4096 x 4096"
#endif
#if STBTE_MAX_LAYERS > 32
#error "Maximum layers allowed is 32"
#endif
#if STBTE_UNDO_BUFFER_COUNT & (STBTE_UNDO_BUFFER_COUNT-1)
#error "Undo buffer size must be a power of 2"
#endif

#if STBTE_MAX_PROPERTIES == 0
#define STBTE__NO_PROPS
#endif

#ifdef STBTE__NO_PROPS
#undef STBTE_MAX_PROPERTIES
#define STBTE_MAX_PROPERTIES 1  // so we can declare arrays
#endif
*/

#[derive(Clone, Copy)]
struct Link {
	x: i16,
	y: i16,
}

/*
enum
{
	STBTE__base,
	STBTE__outline,
	STBTE__text,

	STBTE__num_color_aspects,
};

enum
{
	STBTE__idle,
	STBTE__over,
	STBTE__down,
	STBTE__over_down,
	STBTE__selected,
	STBTE__selected_over,
	STBTE__disabled,
	STBTE__num_color_states,
};
*/

enum ColorMode {
	Expander,
	Toolbar,
	ToolbarButton,
	Panel,
	PanelSider,
	PanelSizer,
	Scrollbar,
	Mapsize,
	LayerButton,
	LayerHide,
	LayerLock,
	LayerSolo,
	CategoryButton,

	/*
	STBTE__num_color_modes,
	*/
}

/*
#ifdef STBTE__COLORPICKER
static char *stbte__color_names[] =
{
	"expander", "toolbar", "tool button", "panel",
	"panel c1", "panel c2", "scollbar", "map button",
	"layer", "hide", "lock", "solo",
	"category",
};
#endif // STBTE__COLORPICKER

		// idle,	 over,	  down,	 over&down, selected, sel&over, disabled
static int stbte__color_table[STBTE__num_color_modes][STBTE__num_color_aspects][STBTE__num_color_states] =
{
	{
		{ 0x000000, 0x84987c, 0xdcdca8, 0xdcdca8, 0x40c040, 0x60d060, 0x505050, },
		{ 0xa4b090, 0xe0ec80, 0xffffc0, 0xffffc0, 0x80ff80, 0x80ff80, 0x606060, },
		{ 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x909090, },
	}, {
		{ 0x808890, 0x606060, 0x606060, 0x606060, 0x606060, 0x606060, 0x606060, },
		{ 0x605860, 0x606060, 0x606060, 0x606060, 0x606060, 0x606060, 0x606060, },
		{ 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, },
	}, {
		{ 0x3c5068, 0x7088a8, 0x647488, 0x94b4dc, 0x8890c4, 0x9caccc, 0x404040, },
		{ 0x889cb8, 0x889cb8, 0x889cb8, 0x889cb8, 0x84c4e8, 0xacc8ff, 0x0c0c08, },
		{ 0xbcc4cc, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x707074, },
	}, {
		{ 0x403848, 0x403010, 0x403010, 0x403010, 0x403010, 0x403010, 0x303024, },
		{ 0x68546c, 0xc08040, 0xc08040, 0xc08040, 0xc08040, 0xc08040, 0x605030, },
		{ 0xf4e4ff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x909090, },
	}, {
		{ 0xb4b04c, 0xacac60, 0xc0ffc0, 0xc0ffc0, 0x40c040, 0x60d060, 0x505050, },
		{ 0xa0a04c, 0xd0d04c, 0xffff80, 0xffff80, 0x80ff80, 0x80ff80, 0x606060, },
		{ 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x909090, },
	}, {
		{ 0x40c440, 0x60d060, 0xc0ffc0, 0xc0ffc0, 0x40c040, 0x60d060, 0x505050, },
		{ 0x40c040, 0x80ff80, 0x80ff80, 0x80ff80, 0x80ff80, 0x80ff80, 0x606060, },
		{ 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x909090, },
	}, {
		{ 0x9090ac, 0xa0a0b8, 0xbcb8cc, 0xbcb8cc, 0x909040, 0x909040, 0x909040, },
		{ 0xa0a0b8, 0xb0b4d0, 0xa0a0b8, 0xa0a0b8, 0xa0a050, 0xa0a050, 0xa0a050, },
		{ 0x808088, 0x808030, 0x808030, 0x808030, 0x808030, 0x808030, 0x808030, },
	}, {
		{ 0x704c70, 0x885c8c, 0x9c68a4, 0xb870bc, 0xb490bc, 0xb490bc, 0x302828, },
		{ 0x646064, 0xcca8d4, 0xc060c0, 0xa07898, 0xe0b8e0, 0xe0b8e0, 0x403838, },
		{ 0xdccce4, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x909090, },
	}, {
		{ 0x704c70, 0x885c8c, 0x9c68a4, 0xb870bc, 0xb490bc, 0xb490bc, 0x302828, },
		{ 0xb09cb4, 0xcca8d4, 0xc060c0, 0xa07898, 0xe0b8e0, 0xe0b8e0, 0x403838, },
		{ 0xdccce4, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x909090, },
	}, {
		{ 0x646494, 0x888cb8, 0xb0b0b0, 0xb0b0cc, 0x9c9cf4, 0x8888b0, 0x50506c, },
		{ 0x9090a4, 0xb0b4d4, 0xb0b0dc, 0xb0b0cc, 0xd0d0fc, 0xd0d4f0, 0x606060, },
		{ 0xb4b4d4, 0xe4e4ff, 0xffffff, 0xffffff, 0xe0e4ff, 0xececff, 0x909090, },
	}, {
		{ 0x646444, 0x888c64, 0xb0b0b0, 0xb0b088, 0xaca858, 0x88886c, 0x505050, },
		{ 0x88886c, 0xb0b490, 0xb0b0b0, 0xb0b088, 0xd8d898, 0xd0d4b0, 0x606060, },
		{ 0xb4b49c, 0xffffd8, 0xffffff, 0xffffd4, 0xffffdc, 0xffffcc, 0x909090, },
	}, {
		{ 0x906464, 0xb48c8c, 0xd4b0b0, 0xdcb0b0, 0xff9c9c, 0xc88888, 0x505050, },
		{ 0xb47c80, 0xd4b4b8, 0xc4a8a8, 0xdcb0b0, 0xffc0c0, 0xfce8ec, 0x606060, },
		{ 0xe0b4b4, 0xffdcd8, 0xffd8d4, 0xffe0e4, 0xffece8, 0xffffff, 0x909090, },
	}, {
		{ 0x403848, 0x403848, 0x403848, 0x886894, 0x7c80c8, 0x7c80c8, 0x302828, },
		{ 0x403848, 0x403848, 0x403848, 0x403848, 0x7c80c8, 0x7c80c8, 0x403838, },
		{ 0xc8c4c8, 0xffffff, 0xffffff, 0xffffff, 0xe8e8ec, 0xffffff, 0x909090, },
	},
};

#define STBTE_COLOR_TILEMAP_BACKGROUND		0x000000
#define STBTE_COLOR_TILEMAP_BORDER			 0x203060
#define STBTE_COLOR_TILEMAP_HIGHLIGHT		 0xffffff
#define STBTE_COLOR_GRID						  0x404040
#define STBTE_COLOR_SELECTION_OUTLINE1		0xdfdfdf
#define STBTE_COLOR_SELECTION_OUTLINE2		0x303030
#define STBTE_COLOR_TILEPALETTE_OUTLINE	  0xffffff
#define STBTE_COLOR_TILEPALETTE_BACKGROUND  0x000000

#ifndef STBTE_LINK_COLOR
#define STBTE_LINK_COLOR(src,sp,dest,dp)	 0x5030ff
#endif

#ifndef STBTE_LINK_COLOR_DRAWING
#define STBTE_LINK_COLOR_DRAWING				0xff40ff
#endif

#ifndef STBTE_LINK_COLOR_DISALLOWED
#define STBTE_LINK_COLOR_DISALLOWED			0x602060
#endif


// disabled, selected, down, over
static unsigned char stbte__state_to_index[2][2][2][2] =
{
	{
		{ { STBTE__idle	 , STBTE__over			 }, { STBTE__down	 , STBTE__over_down }, },
		{ { STBTE__selected, STBTE__selected_over }, { STBTE__down	 , STBTE__over_down }, },
	},{
		{ { STBTE__disabled, STBTE__disabled		}, { STBTE__disabled, STBTE__disabled  }, },
		{ { STBTE__selected, STBTE__selected_over }, { STBTE__disabled, STBTE__disabled  }, },
	}
};
#define STBTE__INDEX_FOR_STATE(disable,select,down,over) stbte__state_to_index[disable][select][down][over]
#define STBTE__INDEX_FOR_ID(id,disable,select) STBTE__INDEX_FOR_STATE(disable,select,STBTE__IS_ACTIVE(id),STBTE__IS_HOT(id))
*/

const FONT_HEIGHT: usize = 9;

fn init_font() {
	let mut n = 95+16;
	for i in 0..95+16 {
		unsafe {
			FONT_OFFSET[i] = n;
		}
		n += FONT_DATA[i];
	}
}

fn get_char_width(ch: char) -> usize {
	assert!(ch <= 127 as char);
	FONT_DATA[ch as usize - 16] as usize
}

fn get_char_bitmap(ch: char) -> *const i16 {
	assert!(ch <= 127 as char);
	unsafe {
		(&FONT_DATA as *const [i16; 769] as *const i16).offset(FONT_OFFSET[ch as usize-16] as isize)
	}
}
fn text_width(s: &str) -> usize {
	s.chars().map(get_char_width).sum()
}



static mut FONT_OFFSET: [i16; 95+16] = [0i16; 95+16];
static FONT_DATA: [i16; 769] = [
	4,9,6,9,9,9,9,8,9,8,4,9,7,7,7,7,4,2,6,8,6,6,7,3,4,4,8,6,3,6,2,6,6,6,6,6,6,
	6,6,6,6,6,2,3,5,4,5,6,6,6,6,6,6,6,6,6,6,6,6,7,6,7,7,7,6,7,6,6,6,6,7,7,6,6,
	6,4,6,4,7,7,3,6,6,5,6,6,5,6,6,4,5,6,4,7,6,6,6,6,6,6,6,6,6,7,6,6,6,5,2,5,8,
	0,0,0,0,2,253,130,456,156,8,72,184,64,2,125,66,64,160,64,146,511,146,146,
	511,146,146,511,146,511,257,341,297,341,297,341,257,511,16,56,124,16,16,16,
	124,56,16,96,144,270,261,262,136,80,48,224,192,160,80,40,22,14,15,3,448,496,
	496,240,232,20,10,5,2,112,232,452,450,225,113,58,28,63,30,60,200,455,257,
	257,0,0,0,257,257,455,120,204,132,132,159,14,4,4,14,159,132,132,204,120,8,
	24,56,120,56,24,8,32,48,56,60,56,48,32,0,0,0,0,111,111,7,7,0,0,7,7,34,127,
	127,34,34,127,127,34,36,46,107,107,58,18,99,51,24,12,102,99,48,122,79,93,
	55,114,80,4,7,3,62,127,99,65,65,99,127,62,8,42,62,28,28,62,42,8,8,8,62,62,
	8,8,128,224,96,8,8,8,8,8,8,96,96,96,48,24,12,6,3,62,127,89,77,127,62,64,66,
	127,127,64,64,98,115,89,77,71,66,33,97,73,93,119,35,24,28,22,127,127,16,39,
	103,69,69,125,57,62,127,73,73,121,48,1,1,113,121,15,7,54,127,73,73,127,54,
	6,79,73,105,63,30,54,54,128,246,118,8,28,54,99,65,20,20,20,20,65,99,54,28,
	8,2,3,105,109,7,2,30,63,33,45,47,46,124,126,19,19,126,124,127,127,73,73,127,
	54,62,127,65,65,99,34,127,127,65,99,62,28,127,127,73,73,73,65,127,127,9,9,
	9,1,62,127,65,73,121,121,127,127,8,8,127,127,65,65,127,127,65,65,32,96,64,
	64,127,63,127,127,8,28,54,99,65,127,127,64,64,64,64,127,127,6,12,6,127,127,
	127,127,6,12,24,127,127,62,127,65,65,65,127,62,127,127,9,9,15,6,62,127,65,
	81,49,127,94,127,127,9,25,127,102,70,79,73,73,121,49,1,1,127,127,1,1,63,127,
	64,64,127,63,15,31,48,96,48,31,15,127,127,48,24,48,127,127,99,119,28,28,119,
	99,7,15,120,120,15,7,97,113,89,77,71,67,127,127,65,65,3,6,12,24,48,96,65,
	65,127,127,8,12,6,3,6,12,8,64,64,64,64,64,64,64,3,7,4,32,116,84,84,124,120,
	127,127,68,68,124,56,56,124,68,68,68,56,124,68,68,127,127,56,124,84,84,92,
	24,8,124,126,10,10,56,380,324,324,508,252,127,127,4,4,124,120,72,122,122,
	64,256,256,256,506,250,126,126,16,56,104,64,66,126,126,64,124,124,24,56,28,
	124,120,124,124,4,4,124,120,56,124,68,68,124,56,508,508,68,68,124,56,56,124,
	68,68,508,508,124,124,4,4,12,8,72,92,84,84,116,36,4,4,62,126,68,68,60,124,
	64,64,124,124,28,60,96,96,60,28,28,124,112,56,112,124,28,68,108,56,56,108,
	68,284,316,352,320,508,252,68,100,116,92,76,68,8,62,119,65,65,127,127,65,
	65,119,62,8,16,24,12,12,24,24,12,4,
];




















struct TileInfo {
	id: i16,
	category_id: u16,
	category: String,
	layermask: u32,
}

/*
#define MAX_LAYERMASK	 (1 << (8*sizeof(unsigned int)))
*/

type TileData = i16;

const NO_TILE: TileData = -1;

/*
enum
{
	STBTE__panel_toolbar,
	STBTE__panel_colorpick,
	STBTE__panel_info,
	STBTE__panel_layers,
	STBTE__panel_props,
	STBTE__panel_categories,
	STBTE__panel_tiles,

	STBTE__num_panel,
};

enum
{
	STBTE__side_left,
	STBTE__side_right,
	STBTE__side_top,
	STBTE__side_bottom,
};

enum
{
	STBTE__tool_select,
	STBTE__tool_brush,
	STBTE__tool_erase,
	STBTE__tool_rect,
	STBTE__tool_eyedrop,
	STBTE__tool_fill,
	STBTE__tool_link,

	STBTE__tool_showgrid,
	STBTE__tool_showlinks,

	STBTE__tool_undo,
	STBTE__tool_redo,
	// copy/cut/paste aren't included here because they're displayed differently

	STBTE__num_tool,
};

// icons are stored in the 0-31 range of ASCII in the font
static int toolchar[] = { 26,24,25,20,23,22,18, 19,17, 29,28, };

enum
{
	STBTE__propmode_default,
	STBTE__propmode_always,
	STBTE__propmode_never,
};
*/

#[derive(Clone, Copy, PartialEq)]
enum Event {
	Paint,

	// from here down does hittesting
	Tick,
	MouseMove,
	MouseWheel,
	LeftDown,
	LeftUp,
	RightDown,
	RightUp,
}

struct Panel {
	expanded: isize,
	mode: isize,
	// number of rows they've requested for this
	delta_height: isize,
	side: isize,
	width: isize,
	height: isize,
	x0: isize,
	y0: isize,
}

struct ColorRect  {
	x0: usize,
	y0: usize,
	x1: usize,
	y1: usize,
	color: u32,
}

const MAX_DELAYRECT: usize = 256;

struct UI {
	tool: usize, active_event: Event,

	active_id: u32, hot_id: u32, next_hot_id: u32,
	event: Event,
	mx: usize,my: usize, dx: usize,dy: usize,
	/*
	int ms_time;
	int shift, scrollkey;
	int initted;
	int side_extended[2];
	*/
	delayrect: [ColorRect; MAX_DELAYRECT],
	delaycount: usize,
	/*
	int show_grid, show_links;
	int brush_state; // used to decide which kind of erasing
	int eyedrop_x, eyedrop_y, eyedrop_last_layer;
	int pasting, paste_x, paste_y;
	int scrolling, start_x, start_y;
	int last_mouse_x, last_mouse_y;
	*/
	accum_x: usize, accum_y: usize,
	/*
	int linking;
	int dragging;
	int drag_x, drag_y, drag_w, drag_h;
	int drag_offx, drag_offy, drag_dest_x, drag_dest_y;
	int undoing;
	*/
	has_selection: bool, select_x0: usize, select_y0: usize, select_x1: usize, select_y1: usize,
	sx: u32,sy: u32,

	// configurable widths
	x0: usize,y0: usize,x1: usize,y1: usize, left_width: usize, right_width: usize,

	alert_timer: f32,
	alert_msg: String,

/*
	float dt;
	stbte__panel panel[STBTE__num_panel];
	short copybuffer[STBTE_MAX_COPY][STBTE_MAX_LAYERS];
	float copyprops[STBTE_MAX_COPY][STBTE_MAX_PROPERTIES];
#ifdef STBTE_ALLOW_LINK
	stbte__link copylinks[STBTE_MAX_COPY];
#endif
	int copy_src_x, copy_src_y;
	stbte_tilemap *copy_src;
	int copy_width,copy_height,has_copy,copy_has_props;
	*/
}

/*
// there's only one UI system at a time, so we can globalize this
static stbte__ui_t stbte__ui = { STBTE__tool_brush, 0 };

#define STBTE__INACTIVE()	  (stbte__ui.active_id == 0)
#define STBTE__IS_ACTIVE(id)  (stbte__ui.active_id == (id))
#define STBTE__IS_HOT(id)	  (stbte__ui.hot_id	 == (id))
*/

const BUTTON_INTERNAL_SPACING: usize = 2 + (FONT_HEIGHT>>4);
const BUTTON_HEIGHT: usize = FONT_HEIGHT + 2 * BUTTON_INTERNAL_SPACING;

struct Layer {
	name: String,
	locked: Lock,
	hidden: bool,
}

#[derive(PartialEq)]
enum Lock {
	Unlocked,
	Protected,
	Locked,
}

struct Tilemap {
	data: [[[TileData; MAX_LAYERS]; MAX_TILEMAP_X]; MAX_TILEMAP_Y],
	props: [[[f32; MAX_PROPERTIES]; MAX_TILEMAP_X]; MAX_TILEMAP_Y],

	link: [[Link; MAX_TILEMAP_X];MAX_TILEMAP_X],
	linkcount: [[usize; MAX_TILEMAP_X];MAX_TILEMAP_X],

	max_x: usize, max_y: usize, num_layers: isize,
	spacing_x: usize, spacing_y: usize,
	palette_spacing_x: usize, palette_spacing_y: usize,
	scroll_x: usize, scroll_y: usize,

	cur_category: u16, cur_tile: isize, cur_layer: isize,

	categories: [String; MAX_CATEGORIES],
	num_categories: u16, category_scroll: usize,

	tiles: Vec<TileInfo>,
	num_tiles: usize, max_tiles: usize, digits: isize,
	/*
	unsigned char undo_available_valid;
	unsigned char undo_available;
	unsigned char redo_available;
	unsigned char padding;
	*/
	cur_palette_count: usize,
	palette_scroll: usize,

	tileinfo_dirty: bool,
	layerinfo: [Layer; MAX_LAYERS],
	has_layer_names: bool,
	layername_width: usize,
	/*
	int layer_scroll;
	int propmode;
	*/
	solo_layer: isize,
	/*
	int undo_pos, undo_len, redo_len;
	*/
	background_tile: i16,
	id_in_use: [u8; 32768>>3],
	undo_buffer: Vec<i16>,
}

const DEFAULT_CATEGORY: &str = "[unassigned]";

/*
static void stbte__init_gui(void)
{
	int i,n;
	stbte__ui.initted = 1;
	// init UI state
	stbte__ui.show_links = 1;
	for (i=0; i < STBTE__num_panel; ++i) {
		stbte__ui.panel[i].expanded	  = 1; // visible if not autohidden
		stbte__ui.panel[i].delta_height = 0;
		stbte__ui.panel[i].side			= STBTE__side_left;
	}
	stbte__ui.panel[STBTE__panel_toolbar  ].side = STBTE__side_top;
	stbte__ui.panel[STBTE__panel_colorpick].side = STBTE__side_right;

	if (stbte__ui.left_width == 0)
		stbte__ui.left_width = 80;
	if (stbte__ui.right_width == 0)
		stbte__ui.right_width = 80;

	init_font();
}
*/


// #define STBTE__BG(tm,layer) ((layer) == 0 ? (tm)->background_tile : STBTE__NO_TILE)
macro_rules! bg {
	($self: ident, $layer: expr) => {
		if $layer == 0 {
			$self.background_tile
		} else {
			NO_TILE
		}
	};
}

/*
	fn bg(&self, layer: usize) -> i16 {
		if layer == 0 {
			self.background_tile
		} else {
			NO_TILE
		}
	}
*/

impl Tilemap {
	fn new(map_x: usize, map_y: usize, map_layers: usize, spacing_x: usize, spacing_y: usize, max_tiles: usize) -> Tilemap {
		let mut tm: Tilemap = unsafe { mem::uninitialized() };

		assert!(map_layers <= MAX_LAYERS);
		assert!(map_x <= MAX_TILEMAP_X);
		assert!(map_y <= MAX_TILEMAP_Y);

		/* TODO:
		if (!stbte__ui.initted)
			stbte__init_gui();
		*/

		tm.tiles = Vec::with_capacity(max_tiles);
		tm.undo_buffer = Vec::with_capacity(UNDO_BUFFER_BYTES / 2);
		tm.num_layers = map_layers as isize;
		tm.max_x = map_x;
		tm.max_y = map_y;
		tm.spacing_x = spacing_x;
		tm.spacing_y = spacing_y;
		tm.scroll_x = 0;
		tm.scroll_y = 0;
		/*
		self.palette_scroll = 0;
		self.palette_spacing_x = spacing_x+1;
		self.palette_spacing_y = spacing_y+1;
		self.cur_category = -1;
		self.cur_tile = 0;
		self.solo_layer = -1;
		self.undo_len = 0;
		self.redo_len = 0;
		self.undo_pos = 0;
		self.category_scroll = 0;
		self.layer_scroll = 0;
		self.propmode = 0;
		self.has_layer_names = 0;
		self.layername_width = 0;
		self.undo_available_valid = 0;

		for (i=0; i < self.num_layers; ++i) {
			self.layerinfo[i].hidden = 0;
			self.layerinfo[i].locked = STBTE__unlocked;
			self.layerinfo[i].name	= 0;
		}

		self.background_tile = STBTE__NO_TILE;
		stbte_clear_map(tm);

		self.max_tiles = max_tiles;
		self.num_tiles = 0;
		for (i=0; i < 32768/8; ++i)
			self.id_in_use[i] = 0;
		self.tileinfo_dirty = true;
		return tm;
		*/

		tm
	}


	fn set_background_tile(&mut self, id: i16) {
		assert!(id >= -1);
		for i in 0..MAX_TILEMAP_X * MAX_TILEMAP_Y {
			if self.data[0][i][0] == -1 {
				self.data[0][i][0] = id;
			}
		}
		self.background_tile = id;
	}

	fn set_spacing(&mut self, spacing_x: usize, spacing_y: usize, palette_spacing_x: usize, palette_spacing_y: usize) {
		self.spacing_x = spacing_x;
		self.spacing_y = spacing_y;
		self.palette_spacing_x = palette_spacing_x;
		self.palette_spacing_y = palette_spacing_y;
	}

	fn define_tile(&mut self, id: u16, layermask: u32, category: Option<String>) {
		let category = category.unwrap_or(DEFAULT_CATEGORY.to_string());
		assert!(id < 32768);
		assert!(self.tiles.len() < self.tiles.capacity());
		assert!((self.id_in_use[id as usize >> 3]&(1<<(id&7))) == 0);

		self.id_in_use[id as usize >> 3] |= 1 << (id&7);

		self.tiles.push(TileInfo {
			category,
			layermask,
			id: id as i16,
			category_id: 0,
		});

		self.num_tiles += 1;
		self.tileinfo_dirty = true;
	}

	fn set_layername(&mut self, layer: usize, layername: String) {
		let width = text_width(&layername);
		self.layerinfo[layer].name = layername;
		self.has_layer_names = true;
		self.layername_width = ::std::cmp::max(width, self.layername_width);
	}

	fn dimensions(&self) -> (usize, usize) {
		(self.max_x, self.max_y)
	}

	fn set_dimensions(&mut self, map_x: usize, map_y: usize) {
		assert!(map_x <= MAX_TILEMAP_X);
		assert!(map_y <= MAX_TILEMAP_Y);
		self.max_x = map_x;
		self.max_y = map_y;
	}

	fn tile(&mut self, x: usize, y: usize) -> &mut [TileData; MAX_LAYERS] {
		&mut self.data[y][x]
	}

	fn properties(&mut self, x: usize, y: usize) -> &mut [f32; MAX_PROPERTIES] {
		&mut self.props[y][x]
	}

	fn set_property(&mut self, x: usize, y: usize, n: usize, val: f32) {
		self.props[y][x][n] = val;
	}

	fn clear_map(&mut self) {
		for i in 0..MAX_TILEMAP_X * MAX_TILEMAP_Y {
			self.data[0][i][0] = self.background_tile;

			for j in 1..self.num_layers as usize {
				self.data[0][i][j] = NO_TILE;
			}
			for j in 0..MAX_PROPERTIES {
				self.props[0][i][j] = 0.0;
			}

			self.link[0][i].x = -1;
			self.link[0][i].y = -1;
			self.linkcount[0][i] = 0;
		}
	}


	fn link(&mut self, x: usize, y: usize) -> Option<Link> {
		assert!(x < self.max_x && y < self.max_y);
		let link = self.link[y][x];
		if link.x >= 0 {
			if !ALLOW_LINK(&self.data[y][x], &self.props[y][x], &self.data[link.y as usize][link.x as usize], &self.props[link.y as usize][link.x as usize]) {
				None
			} else {
				Some(link)
			}
		} else {
			None
		}
	}

	fn set_tile(&mut self, x: usize, y: usize, layer: usize, tile: i16) {
		/* TODO:
		STBTE_ASSERT(x >= 0 && x < self.max_x && y >= 0 && y < self.max_y);
		STBTE_ASSERT(layer >= 0 && layer < self.num_layers);
		STBTE_ASSERT(tile >= -1 && tile < 32768);
		if (x < 0 || x >= STBTE_MAX_TILEMAP_X || y < 0 || y >= STBTE_MAX_TILEMAP_Y)
			return;
		if (layer < 0 || layer >= self.num_layers || tile < -1)
			return;
			*/
		self.data[y][x][layer] = tile;
	}


	fn choose_category(&mut self, category: u16) {
		let mut n = 0;
		self.cur_category = category;
		for i in 0..self.num_tiles {
			if self.tiles[i].category_id == category || category == 0xFFFF {
				n += 1;
			}
		}
		self.cur_palette_count = n;
		self.palette_scroll = 0;
	}


	fn prepare_tileinfo(&mut self) {
		if self.tileinfo_dirty {
			self.compute_tileinfo();
		}
	}
	fn compute_tileinfo(&mut self) {

		self.num_categories=0;

		/* TODO:
		let mut n = 0;
		for (i=0; i < self.num_tiles; ++i) {
			stbte__tileinfo *t = &self.tiles[i];
			// find category
			for (j=0; j < self.num_categories; ++j) {
				if t.category == self.categories[j]) {
					goto found;
				}
			}
			self.categories[j] = t->category;
			++self.num_categories;
		found:
			t.category_id = j as u16;
		}
		*/

		// currently number of categories can never decrease because you
		// can't remove tile definitions, but let's get it right anyway
		if self.cur_category > self.num_categories {
			self.cur_category = 0xFFFF;
		}

		let cur = self.cur_category;
		self.choose_category(cur);

		self.tileinfo_dirty = false;
	}

	fn set_link(&mut self, src_x: usize, src_y: usize, dest_x: i16, dest_y: i16) {
		self._set_link(src_x, src_y, dest_x, dest_y, UndoMode::None)
	}

	fn _set_link(&mut self, src_x: usize, src_y: usize, dest_x: i16, dest_y: i16, undo: UndoMode) {
		assert!(src_x < MAX_TILEMAP_X && src_y < MAX_TILEMAP_Y);
		let a = &mut self.link[src_y][src_x];
		// check if it's a do nothing
		if a.x == dest_x && a.y == dest_y {
			return;
		}

		if undo != UndoMode::None {
		/* TODO:
			if undo == UndoMode::Block) {
				self.begin_undo();
			}
			self.undo_record_prop(src_x, src_y, -1, a.x, a.y);
			if undo == UndoMode::Block) {
				self.end_undo();
			}
		*/
		}

		// check if there's an existing link
		if a.x >= 0 {
			// decrement existing link refcount
			assert!(self.linkcount[a.y as usize][a.x as usize] > 0);
			self.linkcount[a.y as usize][a.x as usize] -= 1;
		}
		// increment new dest
		if dest_x >= 0 {
			self.linkcount[dest_y as usize][dest_x as usize] += 1;
		}
		a.x = dest_x;
		a.y = dest_y;
	}

	fn compute_digits(&mut self) {
		self.digits = if self.max_x >= 1000 || self.max_y >= 1000 {
			4
		} else if self.max_x >= 100 || self.max_y >= 100 {
			3
		} else {
			2
		};
	}
}


#[derive(PartialEq)]
enum UndoMode {
	None,
	Record,
	Block,
}


	// returns an array of map_layers shorts. each short is either
	// one of the tile_id values from define_tile, or STBTE_EMPTY





	///////////////////////////////////////////////////////////////////////////////////////////////////


trait Draw {
	fn rect(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32);
	fn line(&mut self, mut x0: usize, mut y0: usize, mut x1: usize, mut y1: usize, color: u32) {
		if x1 < x0 { mem::swap(&mut x1, &mut x0); }
		if y1 < y0 { mem::swap(&mut y1, &mut y0); }
		self.rect(x0, y0, x1+1, y1+1, color);
	}

	fn link(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
		self.line(x0,y0, x0,y1, color);
		self.line(x0,y1, x1,y1, color);
	}

	fn frame(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32)  {
		self.rect(x0,y0,x1-1,y0+1,color);
		self.rect(x1-1,y0,x1,y1-1,color);
		self.rect(x0+1,y1-1,x1,y1,color);
		self.rect(x0,y0+1,x0+1,y1,color);
	}

	fn halfframe(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
		self.rect(x0,y0,x1,y0+1,color);
		self.rect(x0,y0+1,x0+1,y1,color);
	}

	fn bitmask_as_columns(&mut self, x: usize, y: usize, mut bitmask: i16, color: u32) {
		let mut start_i: isize = -1;
		let mut i: usize = 0;
		while bitmask != 0 {
			if bitmask & (1 << i as i16) != 0 {
				if start_i < 0 {
					start_i = i as isize;
				}
			} else if start_i >= 0 {
				self.rect(x, y + start_i as usize, x+1, y+i, color);
				start_i = -1;
				bitmask &= !((1<<i)-1); // clear all the old bits; we don't clear them as we go to save code
			}
			i += 1;
		}
	}

	fn bitmap(&mut self, x: usize, y: usize, w: usize, bitmap: *const i16, color: u32) {
		for i in 0..w {
			unsafe {
				self.bitmask_as_columns(x+i, y, *bitmap.offset(i as isize), color);
			}
		}
	}


	fn text(&mut self, x: usize, y: usize, s: &str, w: usize, color: u32) {
		self.text_core(x,y,s,w,color, false);
	}
	fn text_core(&mut self, mut x: usize, y: usize, s: &str, w: usize, color: u32, digitspace: bool) {
		let x_end = x + w;
		for c in s.chars() {
			let mut cw = get_char_width(c);
			if x + cw > x_end {
				break;
			}
			self.bitmap(x, y, cw, get_char_bitmap(c), color);
			if digitspace && c == ' ' {
				cw = get_char_width('0');
			}
			x += cw + 1;
		}
	}
}

impl UI {
	fn set_sidewidths(&mut self, left: usize, right: usize) {
		self.left_width  = left;
		self.right_width = right;
	}

	fn set_display(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
		self.x0 = x0;
		self.y0 = y0;
		self.x1 = x1;
		self.y1 = y1;
	}


	fn draw_frame_delayed(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
		if self.delaycount < MAX_DELAYRECT {
			let r = ColorRect { x0, y0, x1, y1, color };
			self.delayrect[self.delaycount] = r;
			self.delaycount += 1
		}
	}

/* TODO:
	fn flush_delay(&mut self) {
		for i in 0..self.delaycount {
			let r = self.delayrect[i];
			self.draw_frame(r.x0, r.y0, r.x1, r.y1, r.color);
		}
		self.delaycount = 0;
	}
*/

	fn activate(&mut self, id: u32) {
		self.active_id = id;
		self.active_event = self.event;
		self.accum_x = 0;
		self.accum_y = 0;
	}

	fn hittest(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, id: u32) -> bool {
		let over =
			self.mx >= x0 && self.my >= y0 &&
			self.mx <  x1 && self.my <  y1;
		if over && self.event != Event::Tick {
			self.next_hot_id = id;
		}
		over
	}
}

/* TODO:
	static void stbte__draw_box(int x0, int y0, int x1, int y1, int colormode, int colorindex)
	{
		stbte__draw_rect (x0,y0,x1,y1, stbte__color_table[colormode][STBTE__base	][colorindex]);
		stbte__draw_frame(x0,y0,x1,y1, stbte__color_table[colormode][STBTE__outline][colorindex]);
	}

	static void stbte__draw_textbox(int x0, int y0, int x1, int y1, char *text, int xoff, int yoff, int colormode, int colorindex)
	{
		stbte__draw_box(x0,y0,x1,y1,colormode,colorindex);
		stbte__draw_text(x0+xoff,y0+yoff, text, x1-x0-xoff-1, stbte__color_table[colormode][STBTE__text][colorindex]);
	}
	*/


enum Change {
	Begin,
	End,
	Change,
}

trait UI_: Draw {
	fn event(&self) -> Event;
	fn active_event(&self) -> Event;
	fn hot_id(&self) -> u16;
	fn active_id(&self) -> u16;
	fn activate(&mut self, id: u16);
	fn hittest(&mut self, usize, usize, usize, usize, u16) -> bool;

	fn mx(&mut self) -> &mut isize;
	fn sx(&mut self) -> &mut isize;

	fn index_for_id(&mut self, id: u16, disable: bool, select: bool) -> usize;

	fn draw_box(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, colormode: ColorMode, colorindex: usize);
	fn draw_textbox(&mut self,
		x0: usize, y0: usize, x1: usize, y1: usize,
		text: &str, xoff: usize, yoff: usize, colormode: ColorMode, colorindex: usize);


	fn inactive(&self) -> bool { self.active_id() == 0 }
	fn is_active(&self, id: u16) -> bool { self.active_id() == id }
	fn is_hot(&self, id: u16) -> bool { self.hot_id() == id }

	fn button_core(&mut self, id: u16) -> Option<bool> {
		match self.event() {
			Event::LeftDown if self.is_hot(id) && self.inactive() => self.activate(id),
			Event::LeftUp if self.is_active(id) && self.is_hot(id) => {
				self.activate(0);
				return Some(true);
			}
			Event::RightDown if self.is_hot(id) && self.inactive() => { self.activate(id) }
			Event::RightUp if self.is_active(id) && self.is_hot(id) => { 
				self.activate(0);
				return None;
			}
			_ => (),
		}
		Some(false)
	}

	fn button(&mut self, colormode: ColorMode, label: &str, x: usize, y: usize, textoff: usize, width: usize, id: u16, toggled: bool, disabled: bool) -> bool {
		let x0 = x;
		let y0 = y;
		let x1 = x + width;
		let y1 = y + BUTTON_HEIGHT;

		let s = BUTTON_INTERNAL_SPACING;

		let _over = !disabled && self.hittest(x0,y0,x1,y1, id);
			
		if self.event() == Event::Paint {
			let idx = self.index_for_id(id, disabled, toggled);
			self.draw_textbox(x0,y0,x1,y1, label,s+textoff,s, colormode, idx);
		}

		disabled && self.button_core(id) == Some(true)
	}

	fn button_icon(&mut self, colormode: ColorMode, ch: char, x: usize, y: usize, width: usize, id: u16, toggled: bool, disabled: bool) -> bool {
		let x0=x;
		let y0=y;
		let x1=x+width;
		let y1=y+BUTTON_HEIGHT;
		let s = BUTTON_INTERNAL_SPACING;

		let _over = self.hittest(x0,y0,x1,y1,id);
			
		if self.event() == Event::Paint {
			let label = format!("{}", ch);
			let pad = (9 - get_char_width(ch)) / 2;
			let idx = self.index_for_id(id, disabled, toggled);
			self.draw_textbox(x0,y0,x1,y1, &label, s+pad,s, colormode, idx);
		}

		disabled && self.button_core(id) == Some(true)
	}

	fn minibutton(&mut self, colormode: ColorMode, x: usize, y: usize, ch: char, id: u16) -> Option<bool> {
		let x0 = x;
		let y0 = y;
		let x1 = x+8;
		let y1 = y+7;
		let _over = self.hittest(x0,y0,x1,y1, id);
		if self.event() == Event::Paint {
			let s = format!("{}", ch);
			let idx = self.index_for_id(id, false, false);
			self.draw_textbox(x0,y0,x1,y1, &s, 1,0,colormode, idx);
		}
		self.button_core(id)
	}

	fn layerbutton(&mut self, x: usize, y: usize, ch: char, id: u16, toggled: bool, disabled: bool, colormode: ColorMode) -> Option<bool> {
		let x0 = x;
		let y0 = y;
		let x1 = x+10;
		let y1 = y+11;
		let _over = !disabled && self.hittest(x0,y0,x1,y1, id);
		if self.event() == Event::Paint {
			let s = format!("{}", ch);
			let off = (9-get_char_width(ch))/2;
			let idx = self.index_for_id(id,disabled,toggled);
			self.draw_textbox(x0,y0,x1,y1, &s, off+1,2, colormode, idx);
		}

		if disabled {
			Some(false)
		} else {
			self.button_core(id)
		}
	}
	fn microbutton(&mut self, x: usize, y: usize, size: usize, id: u16, colormode: ColorMode) -> Option<bool> {
		let x0 = x;
		let y0 = y;
		let x1 = x+size;
		let y1 = y+size;
		let _over = self.hittest(x0,y0,x1,y1, id);
		if self.event() == Event::Paint {
			let idx = self.index_for_id(id,false,false);
			self.draw_box(x0,y0,x1,y1, colormode, idx);
		}
		self.button_core(id)
	}

	fn microbutton_dragger(&mut self, x: usize, y: usize, size: usize, id: u16, pos: &mut isize) -> Option<bool> {
		let x0 = x;
		let y0 = y;
		let x1 = x+size;
		let y1 = y+size;
		let _over = self.hittest(x0,y0,x1,y1,id);
		match self.event() {
			Event::Paint => {
				let idx = self.index_for_id(id, false, false);
				self.draw_box(x0,y0,x1,y1, ColorMode::Expander, idx);
			}
			Event::LeftDown => {
				if self.is_hot(id) && self.inactive() {
					self.activate(id);
					*self.sx() = *self.mx() - *pos;
				}
			}
			Event::MouseMove => {
				if self.is_active(id) && self.active_event() == Event::LeftDown {
					*pos = *self.mx() - *self.sx();
				}
			}
			Event::LeftUp => {
				if self.is_active(id) {
					self.activate(0);
				}
			}
			_ => return self.button_core(id),
		}
		Some(false)
	}

	fn category_button(&mut self, label: &str, x: usize, y: usize, width: usize, id: u16, toggled: bool) -> bool {
		let x0=x;
		let y0=y;
		let x1=x+width;
		let y1=y+BUTTON_HEIGHT;
		let s = BUTTON_INTERNAL_SPACING;

		let _over = self.hittest(x0,y0,x1,y1,id);
			
		if self.event() == Event::Paint {
			let idx = self.index_for_id(id, false,toggled);
			self.draw_textbox(x0,y0,x1,y1, label, s,s, ColorMode::CategoryButton, idx);
		}

		self.button_core(id) == Some(true)
	}

	// returns -1 if value changes, 1 at end of drag
	fn slider(&mut self, x0: usize, w: usize, y: usize, range: isize, value: &mut isize, id: u16) -> Option<Change> {
		let x1 = x0+w;
		let pos = (*value as isize * w as isize) / (range+1);
		let _over = self.hittest(x0,y-2,x1,y+3,id);
		let ev = self.event();
		let leftdown = self.event() == Event::LeftDown;
		match ev {
			Event::Paint => {
				self.rect(x0,y,x1,y+1, 0x808080);
				self.rect(x0+pos as usize -1,y-1,x0+pos as usize +2,y+2, 0xffffff);
			}
			Event::LeftDown |
			Event::MouseMove => {
				let ev = if leftdown && self.is_hot(id) && self.inactive() {
					self.activate(id);
					Change::Begin
				} else {
					Change::Change
				};
				if self.is_active(id) {
					let v = (*self.mx() - x0 as isize) * (range+1)/w as isize;
					*value = if v < 0 { 0 } else if v > range { range } else { v };
					return Some(ev);
				}
			}
			Event::LeftUp => {
				if self.is_active(id) {
					self.activate(0);
					return Some(Change::End);
				}
			}
			_ => (),
		}
		None
	}

	fn float_control(&mut self, x0: usize, y0: usize, w: usize, minv: f32, maxv: f32, scale: f32, fmt: &str, value: &mut f32, colormode: ColorMode, id: u16) -> Option<Change> {
		let x1 = x0+w;
		let y1 = y0+11;
		let _over = self.hittest(x0,y0,x1,y1, id);
		match self.event() {
		/* TODO:
			case STBTE__paint: {
				char text[32];
				sprintf(text, fmt ? fmt : "%6.2f", *value);
				stbte__draw_textbox(x0,y0,x1,y1, text, 1,2, colormode, STBTE__INDEX_FOR_ID(id,0,0));
				break;
			}
			*/
			Event::LeftDown | Event::RightDown => {
				if self.is_hot(id) && self.inactive() {
					self.activate(id);
					return Some(Change::Begin);
				}
			}
			Event::LeftUp | Event::RightUp => {
				if self.is_active(id) {
					self.activate(0);
					return Some(Change::End);
				}
			}
			Event::MouseMove if self.is_active(id) => {
			/* TODO:
				float v = *value, delta;
				int ax = stbte__ui.accum_x/FLOAT_CONTROL_GRANULARITY;
				int ay = stbte__ui.accum_y/FLOAT_CONTROL_GRANULARITY;
				stbte__ui.accum_x -= ax*FLOAT_CONTROL_GRANULARITY;
				stbte__ui.accum_y -= ay*FLOAT_CONTROL_GRANULARITY;
				let delta = if self.shift()) {
					if self.active_event() == Event::LeftDown) {
						ax * 16.0f + ay
					} else {
						ax / 16.0f + ay / 256.0f
					}
				} else {
					if self.active_event() == Event::LeftDown) {
						ax*10.0f + ay
					} else {
						ax * 0.1f + ay * 0.01f
					}
				};
				v += delta * scale;
				if (v < minv) v = minv;
				if (v > maxv) v = maxv;
				*value = v;
			*/
				return Some(Change::Change);
				}
			_ => (),
		}
		None
	}
}
/* TODO:

	static void stbte__scrollbar(int x, int y0, int y1, int *val, int v0, int v1, int num_vis, int id)
	{
		int over;
		int thumbpos;
		if (v1 - v0 <= num_vis)
			return;

		// generate thumbpos from numvis
		thumbpos = y0+2 + (y1-y0-4) * *val / (v1 - v0 - num_vis);
		if (thumbpos < y0) thumbpos = y0;
		if (thumbpos >= y1) thumbpos = y1;
		over = stbte__hittest(x-1,y0,x+2,y1,id);
		switch (stbte__ui.event) {
			case STBTE__paint:
				stbte__draw_rect(x,y0,x+1,y1, stbte__color_table[STBTE__cscrollbar][STBTE__text][STBTE__idle]);
				stbte__draw_box(x-1,thumbpos-3,x+2,thumbpos+4, STBTE__cscrollbar, STBTE__INDEX_FOR_ID(id,0,0));
				break;
			case STBTE__leftdown:
				if (STBTE__IS_HOT(id) && STBTE__INACTIVE()) {
					// check if it's over the thumb
					stbte__activate(id);
					*val = ((stbte__ui.my-y0) * (v1 - v0 - num_vis) + (y1-y0)/2)/ (y1-y0);
				}
				break;
			case STBTE__mousemove:
				if (STBTE__IS_ACTIVE(id) && stbte__ui.mx >= x-15 && stbte__ui.mx <= x+15)
					*val = ((stbte__ui.my-y0) * (v1 - v0 - num_vis) + (y1-y0)/2)/ (y1-y0);
				break;
			case STBTE__leftup:
				if (STBTE__IS_ACTIVE(id))
					stbte__activate(0);
				break;

		}

		if (*val >= v1-num_vis)
			*val = v1-num_vis;
		if (*val <= v0)
			*val = v0;
	}

	*/


impl UI {
	fn is_single_selection(&self) -> bool {
		self.has_selection
			&& self.select_x0 == self.select_x1
			&& self.select_y0 == self.select_y1
	}
}

struct Region {
	width: isize, height: isize,
	x: isize,y: isize,
	active: bool,
	retracted: f32,
}

/*
	static stbte__region_t stbte__region[4];

	#define STBTE__TOOLBAR_ICON_SIZE	(9+2*2)
	#define STBTE__TOOLBAR_PASTE_SIZE  (34+2*2)

	// This routine computes where every panel goes onscreen: computes
	// a minimum width for each side based on which panels are on that
	// side, and accounts for width-dependent layout of certain panels.
	static void stbte__compute_panel_locations(stbte_tilemap *tm)
	{
		int i, limit, w, k;
		int window_width  = stbte__ui.x1 - stbte__ui.x0;
		int window_height = stbte__ui.y1 - stbte__ui.y0;
		int min_width[STBTE__num_panel]={0,0,0,0,0,0,0};
		int height[STBTE__num_panel]={0,0,0,0,0,0,0};
		int panel_active[STBTE__num_panel]={1,0,1,1,1,1,1};
		int vpos[4] = { 0,0,0,0 };
		stbte__panel *p = stbte__ui.panel;
		stbte__panel *pt = &p[STBTE__panel_toolbar];
	#ifdef STBTE__NO_PROPS
		int props = 0;
	#else
		int props = 1;
	#endif

		for (i=0; i < 4; ++i) {
			stbte__region[i].active = 0;
			stbte__region[i].width = 0;
			stbte__region[i].height = 0;
		}

		// compute number of digits needs for info panel
		stbte__compute_digits(tm);

		// determine which panels are active
		panel_active[STBTE__panel_categories] = self.num_categories != 0;
		panel_active[STBTE__panel_layers	 ] = self.num_layers	  >  1;
	#ifdef STBTE__COLORPICKER
		panel_active[STBTE__panel_colorpick ] = 1;
	#endif

		panel_active[STBTE__panel_props	  ] = props && stbte__is_single_selection();

		// compute minimum widths for each panel (assuming they're on sides not top)
		min_width[STBTE__panel_info		] = 8 + 11 + 7*self.digits+17+7;					// estimate min width of "w:0000"
		min_width[STBTE__panel_colorpick ] = 120;
		min_width[STBTE__panel_tiles	  ] = 4 + self.palette_spacing_x + 5;				// 5 for scrollbar
		min_width[STBTE__panel_categories] = 4 + 42 + 5;										 // 42 is enough to show ~7 chars; 5 for scrollbar
		min_width[STBTE__panel_layers	 ] = 4 + 54 + 30*self.has_layer_names;			 // 2 digits plus 3 buttons plus scrollbar
		min_width[STBTE__panel_toolbar	] = 4 + STBTE__TOOLBAR_PASTE_SIZE;				// wide enough for 'Paste' button
		min_width[STBTE__panel_props	  ] = 80;						  // narrowest info panel

		// compute minimum widths for left & right panels based on the above
		stbte__region[0].width = stbte__ui.left_width;
		stbte__region[1].width = stbte__ui.right_width;

		for (i=0; i < STBTE__num_panel; ++i) {
			if (panel_active[i]) {
				int side = stbte__ui.panel[i].side;
				if (min_width[i] > stbte__region[side].width)
					stbte__region[side].width = min_width[i];
				stbte__region[side].active = 1;
			}
		}

		// now compute the heights of each panel

		// if toolbar at top, compute its size & push the left and right start points down
		if (stbte__region[STBTE__side_top].active) {
			int height = STBTE__TOOLBAR_ICON_SIZE+2;
			pt->x0	  = stbte__ui.x0;
			pt->y0	  = stbte__ui.y0;
			pt->width  = window_width;
			pt->height = height;
			vpos[STBTE__side_left] = vpos[STBTE__side_right] = height;
		} else {
			int num_rows = STBTE__num_tool * ((stbte__region[pt->side].width-4)/STBTE__TOOLBAR_ICON_SIZE);
			height[STBTE__panel_toolbar] = num_rows*13 + 3*15 + 4; // 3*15 for cut/copy/paste, which are stacked vertically
		}

		for (i=0; i < 4; ++i)
			stbte__region[i].y = stbte__ui.y0 + vpos[i];

		for (i=0; i < 2; ++i) {
			int anim = (int) (stbte__region[i].width * stbte__region[i].retracted);
			stbte__region[i].x = (i == STBTE__side_left) ? stbte__ui.x0 - anim : stbte__ui.x1 - stbte__region[i].width + anim;
		}

		// color picker
		height[STBTE__panel_colorpick] = 300;

		// info panel
		w = stbte__region[p[STBTE__panel_info].side].width;
		p[STBTE__panel_info].mode = (w >= 8 + (11+7*self.digits+17)*2 + 4);
		if (p[STBTE__panel_info].mode)
			height[STBTE__panel_info] = 5 + 11*2 + 2 + self.palette_spacing_y;
		else
			height[STBTE__panel_info] = 5 + 11*4 + 2 + self.palette_spacing_y;

		// layers
		limit = 6 + stbte__ui.panel[STBTE__panel_layers].delta_height;
		height[STBTE__panel_layers] = (self.num_layers > limit ? limit : self.num_layers)*15 + 7 + (self.has_layer_names ? 0 : 11) + props*13;

		// categories
		limit = 6 + stbte__ui.panel[STBTE__panel_categories].delta_height;
		height[STBTE__panel_categories] = (self.num_categories+1 > limit ? limit : self.num_categories+1)*11 + 14;
		if (stbte__ui.panel[STBTE__panel_categories].side == stbte__ui.panel[STBTE__panel_categories].side)
			height[STBTE__panel_categories] -= 4;	

		// palette
		k =  (stbte__region[p[STBTE__panel_tiles].side].width - 8) / self.palette_spacing_x;
		if (k == 0) k = 1;
		height[STBTE__panel_tiles] = ((self.num_tiles+k-1)/k) * self.palette_spacing_y + 8;

		// properties panel
		height[STBTE__panel_props] = 9 + STBTE_MAX_PROPERTIES*14;

		// now compute the locations of all the panels
		for (i=0; i < STBTE__num_panel; ++i) {
			if (panel_active[i]) {
				int side = p[i].side;
				if (side == STBTE__side_left || side == STBTE__side_right) {
					p[i].width  = stbte__region[side].width;
					p[i].x0	  = stbte__region[side].x;
					p[i].y0	  = stbte__ui.y0 + vpos[side];
					p[i].height = height[i];
					vpos[side] += height[i];
					if (vpos[side] > window_height) {
						vpos[side] = window_height;
						p[i].height = stbte__ui.y1 - p[i].y0;
					}
				} else {
					; // it's at top, it's already been explicitly set up earlier
				}
			} else {
				// inactive panel
				p[i].height = 0;
				p[i].width  = 0;
				p[i].x0	  = stbte__ui.x1;
				p[i].y0	  = stbte__ui.y1;
			}
		}
	}
*/

// unique identifiers for imgui
enum Ids {
	Map = 1,
	Region,
	// panel background to hide map, and misc controls
	Panel,
	// info data
	Info,
	// toolbar buttons: param is tool number
	ToolbarA,
	ToolbarB,
	// palette selectors: param is tile index
	Palette,
	// category selectors: param is category index
	Categories,
	Layer,								  //
	// layer controls: param is layer
	Solo,
	Hide, Lock,

	// param is panel ID
	Scrollbar,
	// p1 is panel ID, p2 is destination side
	PanelMover,
	// param panel ID
	PanelSizer,

	ScrollbarId,
	ColorpickId,
	PropFlag,
	PropFloat,
	PropInt,
}

// id is:      [          24-bit data : 7-bit identifier ]
// map id is:  [  12-bit y : 12 bit x : 7-bit identifier ]

fn id(n: u32, p: u32) -> u32 {
	n + (p<<7)
}
fn id2(n: u32, p: u32, q: u32) -> u32 {
	id(n, (p<<12) + q)
}
fn idmap(x: u32, y: u32) -> u32 {
	id2(Ids::Map as u32, x, y)
}

impl UI {
	fn activate_map(&mut self, x: u32, y: u32) {
		self.active_id = idmap(x, y);
		self.active_event = self.event;
		self.sx = x;
		self.sy = y;
	}

	fn alert(&mut self, msg: String) {
		self.alert_msg = msg;
		self.alert_timer = 3.0;
	}
}


#[derive(PartialEq)]
enum Erase {
	Brushonly = 0,
	Any = 1,
	All = 2,
}

impl Tilemap {
	fn brush(&mut self, x: usize, y: usize) {
		let result = &mut self.data[y][x] as *mut [TileData; MAX_LAYERS];
		self.brush_predict(result);
	}
	fn brush_predict(&mut self, result: *mut [TileData; MAX_LAYERS]) {
		// FIXME:
		let result = unsafe { result.as_mut().unwrap() };

		let layer_to_paint = self.cur_layer;

		// find lowest legit layer to paint it on, and put it there

		if layer_to_paint < 0 {
			return;
		}

		let ti = &self.tiles[self.cur_tile as usize];

		for i in 0..self.num_layers {
			// check if object is allowed on layer
			if ti.layermask & (1 << i) == 0 {
				continue;
			}

			if i != self.solo_layer {
				// if there's a selected layer, can only paint on that
				if self.cur_layer >= 0 && i != self.cur_layer {
					continue;
				}

				let i = i as usize;

				// if the layer is hidden, we can't see it
				if self.layerinfo[i].hidden {
					continue;
				}

				// if the layer is locked, we can't write to it
				if self.layerinfo[i].locked == Lock::Locked {
					continue;
				}

				// if the layer is non-empty and protected, can't write to it
				if self.layerinfo[i].locked == Lock::Protected && result[i] != bg!(self, i) {
					continue;
				}
			}

			// TODO: stbte__undo_record(tm,x,y,i,self.data[y][x][i]);
			result[i as usize] = ti.id;
			return;
		}

		//stbte__alert("Selected tile not valid on active layer(s)");
	}

	fn erase(&mut self, x: usize, y: usize, allow_any: Option<Erase>) -> Option<Erase> {
		let result = &mut self.data[y][x] as *mut [TileData; MAX_LAYERS];
		self.erase_predict(result, allow_any)
	}

	fn erase_predict(&self, result: *mut [TileData; MAX_LAYERS], allow_any: Option<Erase>) -> Option<Erase> {
		// FIXME:
		let result = unsafe { result.as_mut().unwrap() };

		let ti = if self.cur_tile >= 0 { Some(&self.tiles[self.cur_tile as usize]) } else { None };

		if allow_any == None {
			return None;
		}

		// first check if only one layer is legit
		let mut i = self.cur_layer;
		if self.solo_layer >= 0 {
			i = self.solo_layer;
		}

		// if only one layer is legit, directly process that one for clarity
		if i >= 0 {
			let i = i as usize;
			let bg = if i == 0 { self.background_tile } else { -1 };
			if self.solo_layer < 0 {
				// check that we're allowed to write to it
				if self.layerinfo[i].hidden { return None; } 
				if self.layerinfo[i].locked != Lock::Unlocked { return None; } 
			}
			if result[i as usize] == bg {
				return None; // didn't erase anything
			}
			if let Some(ti) = ti {
				if result[i] == ti.id && (i != 0 || ti.id != self.background_tile) {
					// TODO: stbte__undo_record(tm,x,y,i,self.data[y][x][i]);
					result[i] = bg;
					return Some(Erase::Brushonly);
				}
			}
			if allow_any == Some(Erase::Any) {
				// TODO: stbte__undo_record(tm,x,y,i,self.data[y][x][i]);
				result[i] = bg;
				return allow_any;
			}
			return None;
		}

		// if multiple layers are legit, first scan all for brush data

		if let Some(ti) = ti {
			if allow_any != Some(Erase::All) {
				let mut i = self.num_layers - 1;
				while { let is = i >= 0; i -= 1; is } {
					let i = i as usize;
					if result[i] != ti.id{
						continue;
					}
					if self.layerinfo[i].locked != Lock::Unlocked || self.layerinfo[i].hidden{
						continue;
					}
					if i == 0 && result[i] == self.background_tile{
						return None;
					}
					// TODO: stbte__undo_record(tm,x,y,i,self.data[y][x][i]);
					result[i] = bg!(self, i);
					return Some(Erase::Brushonly);
				}
			}
		}

		if allow_any != Some(Erase::Any) && allow_any != Some(Erase::All) {
			return None;
		}

		// apply layer filters, erase from top
		let mut i = self.num_layers-1;
		while { let is = i >= 0; i -= 1; is} {
			let i = i as usize;
			if result[i] < 0 {
				continue;
			}
			if self.layerinfo[i].locked != Lock::Unlocked || self.layerinfo[i].hidden {
				continue;
			}
			if i == 0 && result[i] == self.background_tile {
				return None;
			}
			// TODO: stbte__undo_record(tm,x,y,i,self.data[y][x][i]);
			result[i] = bg!(self, i);
			if allow_any != Some(Erase::All) {
				return Some(Erase::Any);
			}
		}

		if allow_any == Some(Erase::All) {
			allow_any
		} else {
			None
		}
	}
}

/*
	static int stbte__find_tile(stbte_tilemap *tm, int tile_id)
	{
		int i;
		for (i=0; i < self.num_tiles; ++i)
			if (self.tiles[i].id == tile_id)
				return i;
		stbte__alert("Eyedropped tile that isn't in tileset");
		return -1;
	}

	static void stbte__eyedrop(stbte_tilemap *tm, int x, int y)
	{
		int i,j;

		// flush eyedropper state
		if (stbte__ui.eyedrop_x != x || stbte__ui.eyedrop_y != y) {
			stbte__ui.eyedrop_x = x;
			stbte__ui.eyedrop_y = y;
			stbte__ui.eyedrop_last_layer = self.num_layers;
		}

		// if only one layer is active, query that
		i = self.cur_layer;
		if (self.solo_layer >= 0)
			i = self.solo_layer;
		if (i >= 0) {
			if (self.data[y][x][i] == STBTE__NO_TILE)
				return;
			self.cur_tile = stbte__find_tile(tm, self.data[y][x][i]);
			return;
		}

		// if multiple layers, continue from previous
		i = stbte__ui.eyedrop_last_layer;
		for (j=0; j < self.num_layers; ++j) {
			if (--i < 0)
				i = self.num_layers-1;
			if (self.layerinfo[i].hidden)
				continue;
			if (self.data[y][x][i] == STBTE__NO_TILE)
				continue;
			stbte__ui.eyedrop_last_layer = i;
			self.cur_tile = stbte__find_tile(tm, self.data[y][x][i]);
			return;
		}
	}

	static int stbte__should_copy_properties(stbte_tilemap *tm)
	{
		int i;
		if (self.propmode == STBTE__propmode_always)
			return 1;
		if (self.propmode == STBTE__propmode_never)
			return 0;
		if (self.solo_layer >= 0 || self.cur_layer >= 0)
			return 0;
		for (i=0; i < self.num_layers; ++i)
			if (self.layerinfo[i].hidden || self.layerinfo[i].locked)
				return 0;
		return 1;
	}

	// compute the result of pasting into a tile non-destructively so we can preview it
	static void stbte__paste_stack(stbte_tilemap *tm, short result[], short dest[], short src[], int dragging)
	{
		int i;

		// special case single-layer
		i = self.cur_layer;
		if (self.solo_layer >= 0)
			i = self.solo_layer;
		if (i >= 0) {
			if (self.solo_layer < 0) {
				// check that we're allowed to write to it
				if (self.layerinfo[i].hidden) return;
				if (self.layerinfo[i].locked == STBTE__locked) return;
				// if protected, dest has to be empty
				if (self.layerinfo[i].locked == STBTE__protected && dest[i] != STBTE__BG(tm,i)) return;
				// if dragging w/o copy, we will try to erase stuff, which protection disallows
				if (dragging && self.layerinfo[i].locked == STBTE__protected)
					return;
			}
			result[i] = dest[i];
			if (src[i] != STBTE__BG(tm,i))
				result[i] = src[i];
			return;
		}

		for (i=0; i < self.num_layers; ++i) {
			result[i] = dest[i];
			if (src[i] != STBTE__NO_TILE)
				if (!self.layerinfo[i].hidden && self.layerinfo[i].locked != STBTE__locked)
					if (self.layerinfo[i].locked == STBTE__unlocked || (!dragging && dest[i] == STBTE__BG(tm,i)))
						result[i] = src[i];
		}
	}

	// compute the result of dragging away from a tile
	static void stbte__clear_stack(stbte_tilemap *tm, short result[])
	{
		int i;
		// special case single-layer
		i = self.cur_layer;
		if (self.solo_layer >= 0)
			i = self.solo_layer;
		if (i >= 0)
			result[i] = STBTE__BG(tm,i);
		else
			for (i=0; i < self.num_layers; ++i)
				if (!self.layerinfo[i].hidden && self.layerinfo[i].locked == STBTE__unlocked)
					result[i] = STBTE__BG(tm,i);
	}

	// check if some map square is active
	#define STBTE__IS_MAP_ACTIVE()  ((stbte__ui.active_id & 127) == STBTE__map)
	#define STBTE__IS_MAP_HOT()	  ((stbte__ui.hot_id & 127) == STBTE__map)

	static void stbte__fillrect(stbte_tilemap *tm, int x0, int y0, int x1, int y1, int fill)
	{
		int i,j;
		int x=x0,y=y0;

		stbte__begin_undo(tm);
		if (x0 > x1) i=x0,x0=x1,x1=i;
		if (y0 > y1) j=y0,y0=y1,y1=j;
		for (j=y0; j <= y1; ++j)
			for (i=x0; i <= x1; ++i)
				if (fill)
					stbte__brush(tm, i,j);
				else
					stbte__erase(tm, i,j,STBTE__erase_any);
		stbte__end_undo(tm);
		// suppress warning from brush
		stbte__ui.alert_msg = 0;
	}

	static void stbte__select_rect(stbte_tilemap *tm, int x0, int y0, int x1, int y1)
	{
		stbte__ui.has_selection = 1;
		stbte__ui.select_x0 = (x0 < x1 ? x0 : x1);
		stbte__ui.select_x1 = (x0 < x1 ? x1 : x0);
		stbte__ui.select_y0 = (y0 < y1 ? y0 : y1);
		stbte__ui.select_y1 = (y0 < y1 ? y1 : y0);
	}

	static void stbte__copy_properties(float *dest, float *src)
	{
		int i;
		for (i=0; i < STBTE_MAX_PROPERTIES; ++i)
			dest[i] = src[i];
	}

	static void stbte__copy_cut(stbte_tilemap *tm, int cut)
	{
		int i,j,n,w,h,p=0;
		int copy_props = stbte__should_copy_properties(tm);
		if (!stbte__ui.has_selection)
			return;
		w = stbte__ui.select_x1 - stbte__ui.select_x0 + 1;
		h = stbte__ui.select_y1 - stbte__ui.select_y0 + 1;
		if (STBTE_MAX_COPY / w < h) {
			stbte__alert("Selection too large for copy buffer, increase STBTE_MAX_COPY");
			return;
		}

		for (i=0; i < w*h; ++i)
			for (n=0; n < self.num_layers; ++n)
				stbte__ui.copybuffer[i][n] = STBTE__NO_TILE;

		if (cut)
			stbte__begin_undo(tm);
		for (j=stbte__ui.select_y0; j <= stbte__ui.select_y1; ++j) {
			for (i=stbte__ui.select_x0; i <= stbte__ui.select_x1; ++i) {
				for (n=0; n < self.num_layers; ++n) {
					if (self.solo_layer >= 0) {
						if (self.solo_layer != n)
							continue;
					} else {
						if (self.cur_layer >= 0)
							if (self.cur_layer != n)
								continue;
						if (self.layerinfo[n].hidden)
							continue;
						if (cut && self.layerinfo[n].locked)
							continue;
					}
					stbte__ui.copybuffer[p][n] = self.data[j][i][n];
					if (cut) {
						stbte__undo_record(tm,i,j,n, self.data[j][i][n]);
						self.data[j][i][n] = (n==0 ? self.background_tile : -1);
					}
				}
				if (copy_props) {
					stbte__copy_properties(stbte__ui.copyprops[p], self.props[j][i]);
	#ifdef STBTE_ALLOW_LINK
					stbte__ui.copylinks[p] = self.link[j][i];
					if (cut)
						stbte__set_link(tm, i,j,-1,-1, STBTE__undo_record);
	#endif
				}
				++p;
			}
		}
		if (cut)
			stbte__end_undo(tm);
		stbte__ui.copy_width = w;
		stbte__ui.copy_height = h;
		stbte__ui.has_copy = 1;
		//stbte__ui.has_selection = 0;
		stbte__ui.copy_has_props = copy_props;
		stbte__ui.copy_src = tm; // used to give better semantics when copying links
		stbte__ui.copy_src_x = stbte__ui.select_x0;
		stbte__ui.copy_src_y = stbte__ui.select_y0;
	}
	*/

use std::ops::Add;

fn in_rect<T: PartialOrd + Add<Output=T>> (x: T, y: T, x0: T, y0: T, w: T, h: T) -> bool {
	x >= x0 && x < x0+w && y >= y0 && y < y0+h
}

/*
	static int stbte__in_src_rect(int x, int y)
	{
		return stbte__in_rect(x,y, stbte__ui.copy_src_x, stbte__ui.copy_src_y, stbte__ui.copy_width, stbte__ui.copy_height);
	}

	static int stbte__in_dest_rect(int x, int y, int destx, int desty)
	{
		return stbte__in_rect(x,y, destx, desty, stbte__ui.copy_width, stbte__ui.copy_height);
	}
	*/

/*
	static void stbte__paste(stbte_tilemap *tm, int mapx, int mapy)
	{
		int w = stbte__ui.copy_width;
		int h = stbte__ui.copy_height;
		int i,j,k,p;
		int x = mapx - (w>>1);
		int y = mapy - (h>>1);
		int copy_props = stbte__should_copy_properties(tm) && stbte__ui.copy_has_props;
		if (stbte__ui.has_copy == 0)
			return;
		stbte__begin_undo(tm);
		p = 0;
		for (j=0; j < h; ++j) {
			for (i=0; i < w; ++i) {
				if (y+j >= 0 && y+j < self.max_y && x+i >= 0 && x+i < self.max_x) {
					// compute the new stack
					short tilestack[STBTE_MAX_LAYERS];
					for (k=0; k < self.num_layers; ++k)
						tilestack[k] = self.data[y+j][x+i][k];
					stbte__paste_stack(tm, tilestack, tilestack, stbte__ui.copybuffer[p], 0);
					// update anything that changed
					for (k=0; k < self.num_layers; ++k) {
						if (tilestack[k] != self.data[y+j][x+i][k]) {
							stbte__undo_record(tm, x+i,y+j,k, self.data[y+j][x+i][k]);
							self.data[y+j][x+i][k] = tilestack[k];
						}
					}
				}
				if (copy_props) {
	#ifdef STBTE_ALLOW_LINK
					// need to decide how to paste a link, so there's a few cases
					int destx = -1, desty = -1;
					stbte__link *link = &stbte__ui.copylinks[p];

					// check if link is within-rect
					if (stbte__in_src_rect(link->x, link->y)) {
						// new link should point to copy (but only if copy is within map)
						destx = x + (link->x - stbte__ui.copy_src_x);
						desty = y + (link->y - stbte__ui.copy_src_y);
					} else if (tm == stbte__ui.copy_src) {
						// if same map, then preserve link unless target is overwritten
						if (!stbte__in_dest_rect(link->x,link->y,x,y)) {
							destx = link->x;
							desty = link->y;
						}
					}
					// this is necessary for offset-copy, but also in case max_x/max_y has changed
					if (destx < 0 || destx >= self.max_x || desty < 0 || desty >= self.max_y)
						destx = -1, desty = -1;
					stbte__set_link(tm, x+i, y+j, destx, desty, STBTE__undo_record);
	#endif
					for (k=0; k < STBTE_MAX_PROPERTIES; ++k) {
						if (self.props[y+j][x+i][k] != stbte__ui.copyprops[p][k])
							stbte__undo_record_prop_float(tm, x+i, y+j, k, self.props[y+j][x+i][k]);
					}
					stbte__copy_properties(self.props[y+j][x+i], stbte__ui.copyprops[p]);
				}
				++p;
			}
		}
		stbte__end_undo(tm);
	}

	static void stbte__drag_update(stbte_tilemap *tm, int mapx, int mapy, int copy_props)
	{
		int w = stbte__ui.drag_w, h = stbte__ui.drag_h;
		int ox,oy,i,deleted=0,written=0;
		short temp[STBTE_MAX_LAYERS];
		short *data = NULL;
		if (!stbte__ui.shift) {
			ox = mapx - stbte__ui.drag_x;
			oy = mapy - stbte__ui.drag_y;
			if (ox >= 0 && ox < w && oy >= 0 && oy < h) {
				deleted=1;
				for (i=0; i < self.num_layers; ++i)
					temp[i] = self.data[mapy][mapx][i];
				data = temp;
				stbte__clear_stack(tm, data);
			}
		}
		ox = mapx - stbte__ui.drag_dest_x;
		oy = mapy - stbte__ui.drag_dest_y;
		// if this map square is in the target drag region
		if (ox >= 0 && ox < w && oy >= 0 && oy < h) {
			// and the src map square is on the map
			if (stbte__in_rect(stbte__ui.drag_x+ox, stbte__ui.drag_y+oy, 0, 0, self.max_x, self.max_y)) {
				written = 1;
				if (data == NULL) {
					for (i=0; i < self.num_layers; ++i)
						temp[i] = self.data[mapy][mapx][i];
					data = temp;
				}
				stbte__paste_stack(tm, data, data, self.data[stbte__ui.drag_y+oy][stbte__ui.drag_x+ox], !stbte__ui.shift);
				if (copy_props) {
					for (i=0; i < STBTE_MAX_PROPERTIES; ++i) {
						if (self.props[mapy][mapx][i] != self.props[stbte__ui.drag_y+oy][stbte__ui.drag_x+ox][i]) {
							stbte__undo_record_prop_float(tm, mapx, mapy, i, self.props[mapy][mapx][i]);
							self.props[mapy][mapx][i] = self.props[stbte__ui.drag_y+oy][stbte__ui.drag_x+ox][i];
						}
					}
				}
			}
		}
		if (data) {
			for (i=0; i < self.num_layers; ++i) {
				if (self.data[mapy][mapx][i] != data[i]) {
					stbte__undo_record(tm, mapx, mapy, i, self.data[mapy][mapx][i]);
					self.data[mapy][mapx][i] = data[i];
				}
			}
		}
		#ifdef STBTE_ALLOW_LINK
		if (copy_props) {
			int overwritten=0, moved=0, copied=0;
			// since this function is called on EVERY tile, we can fix up even tiles not
			// involved in the move

			stbte__link *k;
			// first, determine what src link ends up here
			k = &self.link[mapy][mapx]; // by default, it's the one currently here
			if (deleted)					// if dragged away, it's erased
				k = NULL;
			if (written)					// if dragged into, it gets that link
				k = &self.link[stbte__ui.drag_y+oy][stbte__ui.drag_x+ox];

			// now check whether the *target* gets moved or overwritten
			if (k && k->x >= 0) {
				overwritten = stbte__in_rect(k->x, k->y, stbte__ui.drag_dest_x, stbte__ui.drag_dest_y, w, h);
				if (!stbte__ui.shift)
					moved	 = stbte__in_rect(k->x, k->y, stbte__ui.drag_x	  , stbte__ui.drag_y	  , w, h);
				else
					copied	= stbte__in_rect(k->x, k->y, stbte__ui.drag_x	  , stbte__ui.drag_y	  , w, h);
			}

			if (deleted || written || overwritten || moved || copied) {
				// choose the final link value based on the above
				if (k == NULL || k->x < 0)
					stbte__set_link(tm, mapx, mapy, -1, -1, STBTE__undo_record);
				else if (moved || (copied && written)) {
					// if we move the target, we update to point to the new target;
					// or, if we copy the target and the source is part ofthe copy, then update to new target
					int x = k->x + (stbte__ui.drag_dest_x - stbte__ui.drag_x);
					int y = k->y + (stbte__ui.drag_dest_y - stbte__ui.drag_y);
					if (!(x >= 0 && y >= 0 && x < self.max_x && y < self.max_y))
						x = -1, y = -1;
					stbte__set_link(tm, mapx, mapy, x, y, STBTE__undo_record);
				} else if (overwritten) {
					stbte__set_link(tm, mapx, mapy, -1, -1, STBTE__undo_record);
				} else
					stbte__set_link(tm, mapx, mapy, k->x, k->y, STBTE__undo_record);
			}
		}
		#endif
	}

	static void stbte__drag_place(stbte_tilemap *tm, int mapx, int mapy)
	{
		int i,j;
		int copy_props = stbte__should_copy_properties(tm);
		int move_x = (stbte__ui.drag_dest_x - stbte__ui.drag_x);
		int move_y = (stbte__ui.drag_dest_y - stbte__ui.drag_y);
		if (move_x == 0 && move_y == 0)
			return;

		stbte__begin_undo(tm);
		// we now need a 2D memmove-style mover that doesn't
		// overwrite any data as it goes. this requires being
		// direction sensitive in the same way as memmove
		if (move_y > 0 || (move_y == 0 && move_x > 0)) {
			for (j=self.max_y-1; j >= 0; --j)
				for (i=self.max_x-1; i >= 0; --i)
					stbte__drag_update(tm,i,j,copy_props);
		} else {
			for (j=0; j < self.max_y; ++j)
				for (i=0; i < self.max_x; ++i)
					stbte__drag_update(tm,i,j,copy_props);
		}
		stbte__end_undo(tm);

		stbte__ui.has_selection = 1;
		stbte__ui.select_x0 = stbte__ui.drag_dest_x;
		stbte__ui.select_y0 = stbte__ui.drag_dest_y;
		stbte__ui.select_x1 = stbte__ui.select_x0 + stbte__ui.drag_w - 1;
		stbte__ui.select_y1 = stbte__ui.select_y0 + stbte__ui.drag_h - 1;
	}

	static void stbte__tile_paint(stbte_tilemap *tm, int sx, int sy, int mapx, int mapy, int layer)
	{
		int i;
		int id = STBTE__IDMAP(mapx,mapy);
		int x0=sx, y0=sy;
		int x1=sx+self.spacing_x, y1=sy+self.spacing_y;
		int over = stbte__hittest(x0,y0,x1,y1, id);
		short *data = self.data[mapy][mapx];
		short temp[STBTE_MAX_LAYERS];

		if (STBTE__IS_MAP_HOT()) {
			if (stbte__ui.pasting) {
				int ox = mapx - stbte__ui.paste_x;
				int oy = mapy - stbte__ui.paste_y;
				if (ox >= 0 && ox < stbte__ui.copy_width && oy >= 0 && oy < stbte__ui.copy_height) {
					stbte__paste_stack(tm, temp, self.data[mapy][mapx], stbte__ui.copybuffer[oy*stbte__ui.copy_width+ox], 0);
					data = temp;
				}
			} else if (stbte__ui.dragging) {
				int ox,oy;
				for (i=0; i < self.num_layers; ++i)
					temp[i] = self.data[mapy][mapx][i];
				data = temp;

				// if it's in the source area, remove things unless shift-dragging
				ox = mapx - stbte__ui.drag_x;
				oy = mapy - stbte__ui.drag_y;
				if (!stbte__ui.shift && ox >= 0 && ox < stbte__ui.drag_w && oy >= 0 && oy < stbte__ui.drag_h) {
					stbte__clear_stack(tm, temp);
				}

				ox = mapx - stbte__ui.drag_dest_x;
				oy = mapy - stbte__ui.drag_dest_y;
				if (ox >= 0 && ox < stbte__ui.drag_w && oy >= 0 && oy < stbte__ui.drag_h) {
					stbte__paste_stack(tm, temp, temp, self.data[stbte__ui.drag_y+oy][stbte__ui.drag_x+ox], !stbte__ui.shift);
				}
			} else if (STBTE__IS_MAP_ACTIVE()) {
				if (stbte__ui.tool == STBTE__tool_rect) {
					if ((stbte__ui.ms_time & 511) < 380) {
						int ex = ((stbte__ui.hot_id >> 19) & 4095);
						int ey = ((stbte__ui.hot_id >>  7) & 4095);
						int sx = stbte__ui.sx;
						int sy = stbte__ui.sy;

						if (	((mapx >= sx && mapx < ex+1) || (mapx >= ex && mapx < sx+1))
							&& ((mapy >= sy && mapy < ey+1) || (mapy >= ey && mapy < sy+1))) {
							int i;
							for (i=0; i < self.num_layers; ++i)
								temp[i] = self.data[mapy][mapx][i];
							data = temp;
							if (stbte__ui.active_event == STBTE__leftdown)
								stbte__brush_predict(tm, temp);
							else
								stbte__erase_predict(tm, temp, STBTE__erase_any);
						}
					}
				}
			}
		}

		if (STBTE__IS_HOT(id) && STBTE__INACTIVE() && !stbte__ui.pasting) {
			if (stbte__ui.tool == STBTE__tool_brush) {
				if ((stbte__ui.ms_time & 511) < 300) {
					data = temp;
					for (i=0; i < self.num_layers; ++i)
						temp[i] = self.data[mapy][mapx][i];
					stbte__brush_predict(tm, temp);
				}
			}
		}

		{
			i = layer;
			if (i == self.solo_layer || (!self.layerinfo[i].hidden && self.solo_layer < 0))
				if (data[i] >= 0)
					STBTE_DRAW_TILE(x0,y0, (unsigned short) data[i], 0, self.props[mapy][mapx]);
		}
	}

	static void stbte__tile(stbte_tilemap *tm, int sx, int sy, int mapx, int mapy)
	{
		int tool = stbte__ui.tool;
		int x0=sx, y0=sy;
		int x1=sx+self.spacing_x, y1=sy+self.spacing_y;
		int id = STBTE__IDMAP(mapx,mapy);
		int over = stbte__hittest(x0,y0,x1,y1, id);
		switch (stbte__ui.event) {
			case STBTE__paint: {
				if (stbte__ui.pasting || stbte__ui.dragging || stbte__ui.scrolling)
					break;
				if (stbte__ui.scrollkey && !STBTE__IS_MAP_ACTIVE())
					break;
				if (STBTE__IS_HOT(id) && STBTE__IS_MAP_ACTIVE() && (tool == STBTE__tool_rect || tool == STBTE__tool_select)) {
					int rx0,ry0,rx1,ry1,t;
					// compute the center of each rect
					rx0 = x0 + self.spacing_x/2;
					ry0 = y0 + self.spacing_y/2;
					rx1 = rx0 + (stbte__ui.sx - mapx) * self.spacing_x;
					ry1 = ry0 + (stbte__ui.sy - mapy) * self.spacing_y;
					if (rx0 > rx1) t=rx0,rx0=rx1,rx1=t;
					if (ry0 > ry1) t=ry0,ry0=ry1,ry1=t;
					rx0 -= self.spacing_x/2;
					ry0 -= self.spacing_y/2;
					rx1 += self.spacing_x/2;
					ry1 += self.spacing_y/2;
					stbte__draw_frame(rx0-1,ry0-1,rx1+1,ry1+1, STBTE_COLOR_TILEMAP_HIGHLIGHT);
					break;
				}
				if (STBTE__IS_HOT(id) && STBTE__INACTIVE()) {
					stbte__draw_frame(x0-1,y0-1,x1+1,y1+1, STBTE_COLOR_TILEMAP_HIGHLIGHT);
				}
	#ifdef STBTE_ALLOW_LINK
				if (stbte__ui.show_links && self.link[mapy][mapx].x >= 0) {
					int tx = self.link[mapy][mapx].x;
					int ty = self.link[mapy][mapx].y;
					int lx0,ly0,lx1,ly1;
					if (STBTE_ALLOW_LINK(self.data[mapy][mapx], self.props[mapy][mapx],
												self.data[ty  ][tx  ], self.props[ty  ][tx  ]))
					{
						lx0 =  x0 + (self.spacing_x >> 1) - 1;
						ly0 =  y0 + (self.spacing_y >> 1) - 1;
						lx1 = lx0 + (tx - mapx) * self.spacing_x + 2;
						ly1 = ly0 + (ty - mapy) * self.spacing_y + 2;
						stbte__draw_link(lx0,ly0,lx1,ly1,
							STBTE_LINK_COLOR(self.data[mapy][mapx], self.props[mapy][mapx],
													self.data[ty  ][tx  ], self.props[ty  ][tx]));
					}
				}
	#endif
				break;
			}
		}

		if (stbte__ui.pasting) {
			switch (stbte__ui.event) {
				case STBTE__leftdown:
					if (STBTE__IS_HOT(id)) {
						stbte__ui.pasting = 0;
						stbte__paste(tm, mapx, mapy);
						stbte__activate(0);
					}
					break;
				case STBTE__leftup:
					// just clear it no matter what, since they might click away to clear it
					stbte__activate(0);
					break;
				case STBTE__rightdown:
					if (STBTE__IS_HOT(id)) {
						stbte__activate(0);
						stbte__ui.pasting = 0;
					}
					break;
			}
			return;
		}

		if (stbte__ui.scrolling) {
			if (stbte__ui.event == STBTE__leftup) {
				stbte__activate(0);
				stbte__ui.scrolling = 0;
			}
			if (stbte__ui.event == STBTE__mousemove) {
				self.scroll_x += (stbte__ui.start_x - stbte__ui.mx);
				self.scroll_y += (stbte__ui.start_y - stbte__ui.my);
				stbte__ui.start_x = stbte__ui.mx;
				stbte__ui.start_y = stbte__ui.my;
			}
			return;
		}

		// regardless of tool, leftdown is a scrolldrag
		if (STBTE__IS_HOT(id) && stbte__ui.scrollkey && stbte__ui.event == STBTE__leftdown) {
			stbte__ui.scrolling = 1;
			stbte__ui.start_x = stbte__ui.mx;
			stbte__ui.start_y = stbte__ui.my;
			return;
		}

		switch (tool) {
			case STBTE__tool_brush:
				switch (stbte__ui.event) {
					case STBTE__mousemove:
						if (STBTE__IS_MAP_ACTIVE() && over) {
							// don't brush/erase same tile multiple times unless they move away and back @TODO should just be only once, but that needs another data structure
							if (!STBTE__IS_ACTIVE(id)) {
								if (stbte__ui.active_event == STBTE__leftdown)
									stbte__brush(tm, mapx, mapy);
								else
									stbte__erase(tm, mapx, mapy, stbte__ui.brush_state);
								stbte__ui.active_id = id; // switch to this map square so we don't rebrush IT multiple times
							}
						}
						break;
					case STBTE__leftdown:
						if (STBTE__IS_HOT(id) && STBTE__INACTIVE()) {
							stbte__activate(id);
							stbte__begin_undo(tm);
							stbte__brush(tm, mapx, mapy);
						}
						break;
					case STBTE__rightdown:
						if (STBTE__IS_HOT(id) && STBTE__INACTIVE()) {
							stbte__activate(id);
							stbte__begin_undo(tm);
							if (stbte__erase(tm, mapx, mapy, STBTE__erase_any) == STBTE__erase_brushonly)
								stbte__ui.brush_state = STBTE__erase_brushonly;
							else
								stbte__ui.brush_state = STBTE__erase_any;
						}
						break;
					case STBTE__leftup:
					case STBTE__rightup:
						if (STBTE__IS_MAP_ACTIVE()) {
							stbte__end_undo(tm);
							stbte__activate(0);
						}
						break;
				}
				break;

	#ifdef STBTE_ALLOW_LINK
			case STBTE__tool_link:
				switch (stbte__ui.event) {
					case STBTE__leftdown:
						if (STBTE__IS_HOT(id) && STBTE__INACTIVE()) {
							stbte__activate(id);
							stbte__ui.linking = 1;
							stbte__ui.sx = mapx;
							stbte__ui.sy = mapy;
							// @TODO: undo
						}
						break;
					case STBTE__leftup:
						if (STBTE__IS_HOT(id) && STBTE__IS_MAP_ACTIVE()) {
							if ((mapx != stbte__ui.sx || mapy != stbte__ui.sy) &&
									STBTE_ALLOW_LINK(self.data[stbte__ui.sy][stbte__ui.sx], self.props[stbte__ui.sy][stbte__ui.sx],
															self.data[mapy][mapx], self.props[mapy][mapx]))
								stbte__set_link(tm, stbte__ui.sx, stbte__ui.sy, mapx, mapy, STBTE__undo_block);
							else
								stbte__set_link(tm, stbte__ui.sx, stbte__ui.sy, -1,-1, STBTE__undo_block);
							stbte__ui.linking = 0;
							stbte__activate(0);
						}
						break;

					case STBTE__rightdown:
						if (STBTE__IS_ACTIVE(id)) {
							stbte__activate(0);
							stbte__ui.linking = 0;
						}
						break;
				}
				break;
	#endif

			case STBTE__tool_erase:
				switch (stbte__ui.event) {
					case STBTE__mousemove:
						if (STBTE__IS_MAP_ACTIVE() && over)
							stbte__erase(tm, mapx, mapy, STBTE__erase_all);
						break;
					case STBTE__leftdown:
						if (STBTE__IS_HOT(id) && STBTE__INACTIVE()) {
							stbte__activate(id);
							stbte__begin_undo(tm);
							stbte__erase(tm, mapx, mapy, STBTE__erase_all);
						}
						break;
					case STBTE__leftup:
						if (STBTE__IS_MAP_ACTIVE()) {
							stbte__end_undo(tm);
							stbte__activate(0);
						}
						break;
				}
				break;

			case STBTE__tool_select:
				if (STBTE__IS_HOT(id)) {
					switch (stbte__ui.event) {
						case STBTE__leftdown:
							if (STBTE__INACTIVE()) {
								// if we're clicking in an existing selection...
								if (stbte__ui.has_selection) {
									if (  mapx >= stbte__ui.select_x0 && mapx <= stbte__ui.select_x1
										&& mapy >= stbte__ui.select_y0 && mapy <= stbte__ui.select_y1)
									{
										stbte__ui.dragging = 1;
										stbte__ui.drag_x = stbte__ui.select_x0;
										stbte__ui.drag_y = stbte__ui.select_y0;
										stbte__ui.drag_w = stbte__ui.select_x1 - stbte__ui.select_x0 + 1;
										stbte__ui.drag_h = stbte__ui.select_y1 - stbte__ui.select_y0 + 1;
										stbte__ui.drag_offx = mapx - stbte__ui.select_x0;
										stbte__ui.drag_offy = mapy - stbte__ui.select_y0;
									}
								}
								stbte__ui.has_selection = 0; // no selection until it completes
								stbte__activate_map(mapx,mapy);
							}
							break;
						case STBTE__leftup:
							if (STBTE__IS_MAP_ACTIVE()) {
								if (stbte__ui.dragging) {
									stbte__drag_place(tm, mapx,mapy);
									stbte__ui.dragging = 0;
									stbte__activate(0);
								} else {
									stbte__select_rect(tm, stbte__ui.sx, stbte__ui.sy, mapx, mapy);
									stbte__activate(0);
								}
							}
							break;
						case STBTE__rightdown:
							stbte__ui.has_selection = 0;
							break;
					}
				}
				break;

			case STBTE__tool_rect:
				if (STBTE__IS_HOT(id)) {
					switch (stbte__ui.event) {
						case STBTE__leftdown:
							if (STBTE__INACTIVE())
								stbte__activate_map(mapx,mapy);
							break;
						case STBTE__leftup:
							if (STBTE__IS_MAP_ACTIVE()) {
								stbte__fillrect(tm, stbte__ui.sx, stbte__ui.sy, mapx, mapy, 1);
								stbte__activate(0);
							}
							break;
						case STBTE__rightdown:
							if (STBTE__INACTIVE())
								stbte__activate_map(mapx,mapy);
							break;
						case STBTE__rightup:
							if (STBTE__IS_MAP_ACTIVE()) {
								stbte__fillrect(tm, stbte__ui.sx, stbte__ui.sy, mapx, mapy, 0);
								stbte__activate(0);
							}
							break;
					}
				}
				break;


			case STBTE__tool_eyedrop:
				switch (stbte__ui.event) {
					case STBTE__leftdown:
						if (STBTE__IS_HOT(id) && STBTE__INACTIVE())
							stbte__eyedrop(tm,mapx,mapy);
						break;
				}
				break;
		}
	}

	static void stbte__start_paste(stbte_tilemap *tm)
	{
		if (stbte__ui.has_copy) {
			stbte__ui.pasting = 1;
			stbte__activate(STBTE__ID(STBTE__toolbarB,3));
		}
	}

	static void stbte__toolbar(stbte_tilemap *tm, int x0, int y0, int w, int h)
	{
		int i;
		int estimated_width = 13 * STBTE__num_tool + 8+8+ 120+4 - 30;
		int x = x0 + w/2 - estimated_width/2;
		int y = y0+1;

		for (i=0; i < STBTE__num_tool; ++i) {
			int highlight=0, disable=0;
			highlight = (stbte__ui.tool == i);
			if (i == STBTE__tool_undo || i == STBTE__tool_showgrid)
				x += 8;
			if (i == STBTE__tool_showgrid && stbte__ui.show_grid)
				highlight = 1;
			if (i == STBTE__tool_showlinks && stbte__ui.show_links)
				highlight = 1;
			if (i == STBTE__tool_fill)
				continue;
			#ifndef STBTE_ALLOW_LINK
			if (i == STBTE__tool_link || i == STBTE__tool_showlinks)
				disable = 1;
			#endif
			if (i == STBTE__tool_undo && !stbte__undo_available(tm))
				disable = 1;
			if (i == STBTE__tool_redo && !stbte__redo_available(tm))
				disable = 1;
			if (stbte__button_icon(STBTE__ctoolbar_button, toolchar[i], x, y, 13, STBTE__ID(STBTE__toolbarA, i), highlight, disable)) {
				switch (i) {
					case STBTE__tool_eyedrop:
						stbte__ui.eyedrop_last_layer = self.num_layers; // flush eyedropper state
						// fallthrough
					default:
						stbte__ui.tool = i;
						stbte__ui.has_selection = 0;
						break;
					case STBTE__tool_showlinks:
						stbte__ui.show_links = !stbte__ui.show_links;
						break;
					case STBTE__tool_showgrid:
						stbte__ui.show_grid = (stbte__ui.show_grid+1)%3;
						break;
					case STBTE__tool_undo:
						stbte__undo(tm);
						break;
					case STBTE__tool_redo:
						stbte__redo(tm);
						break;
				}
			}
			x += 13;
		}

		x += 8;
		if (stbte__button(STBTE__ctoolbar_button, "cut"  , x, y,10, 40, STBTE__ID(STBTE__toolbarB,0), 0, !stbte__ui.has_selection))
			stbte__copy_cut(tm, 1);
		x += 42;
		if (stbte__button(STBTE__ctoolbar_button, "copy" , x, y, 5, 40, STBTE__ID(STBTE__toolbarB,1), 0, !stbte__ui.has_selection))
			stbte__copy_cut(tm, 0);
		x += 42;
		if (stbte__button(STBTE__ctoolbar_button, "paste", x, y, 0, 40, STBTE__ID(STBTE__toolbarB,2), stbte__ui.pasting, !stbte__ui.has_copy))
			stbte__start_paste(tm);
	}

	#define STBTE__TEXTCOLOR(n)  stbte__color_table[n][STBTE__text][STBTE__idle]

	static int stbte__info_value(const char *label, int x, int y, int val, int digits, int id)
	{
		if (stbte__ui.event == STBTE__paint) {
			int off = 9-stbte__get_char_width(label[0]);
			char text[16];
			sprintf(text, label, digits, val);
			stbte__draw_text_core(x+off,y, text, 999, STBTE__TEXTCOLOR(STBTE__cpanel),1);
		}
		if (id) {
			x += 9+7*digits+4;
			if (stbte__minibutton(STBTE__cmapsize, x,y, '+', STBTE__ID2(id,1,0)))
				val += (stbte__ui.shift ? 10 : 1);
			x += 9;
			if (stbte__minibutton(STBTE__cmapsize, x,y, '-', STBTE__ID2(id,2,0)))
				val -= (stbte__ui.shift ? 10 : 1);
			if (val < 1) val = 1; else if (val > 4096) val = 4096;
		}
		return val;
	}

	static void stbte__info(stbte_tilemap *tm, int x0, int y0, int w, int h)
	{
		int mode = stbte__ui.panel[STBTE__panel_info].mode;
		int s = 11+7*self.digits+4+15;
		int x,y;
		int in_region;

		x = x0+2;
		y = y0+2;
		self.max_x = stbte__info_value("w:%*d",x,y, self.max_x, self.digits, STBTE__ID(STBTE__info,0));
		if (mode)
			x += s;
		else
			y += 11;
		self.max_y = stbte__info_value("h:%*d",x,y, self.max_y, self.digits, STBTE__ID(STBTE__info,1));
		x = x0+2;
		y += 11;
		in_region = (stbte__ui.hot_id & 127) == STBTE__map;
		stbte__info_value(in_region ? "x:%*d" : "x:",x,y, (stbte__ui.hot_id>>19)&4095, self.digits, 0);
		if (mode)
			x += s;
		else
			y += 11;
		stbte__info_value(in_region ? "y:%*d" : "y:",x,y, (stbte__ui.hot_id>> 7)&4095, self.digits, 0);
		y += 15;
		x = x0+2;
		stbte__draw_text(x,y,"brush:",40,STBTE__TEXTCOLOR(STBTE__cpanel));
		if (self.cur_tile >= 0)
			STBTE_DRAW_TILE(x+43,y-3,self.tiles[self.cur_tile].id,1,0);
	}

	static void stbte__layers(stbte_tilemap *tm, int x0, int y0, int w, int h)
	{
		static char *propmodes[3] = {
			"default", "always", "never"
		};
		int num_rows;
		int i, y, n;
		int x1 = x0+w;
		int y1 = y0+h;
		int xoff = 20;
		
		if (self.has_layer_names) {
			int side = stbte__ui.panel[STBTE__panel_layers].side;
			xoff = stbte__region[side].width - 42;
			xoff = (xoff < self.layername_width + 10 ? xoff : self.layername_width + 10);
		}

		x0 += 2;
		y0 += 5;
		if (!self.has_layer_names) {
			if (stbte__ui.event == STBTE__paint) {
				stbte__draw_text(x0,y0, "Layers", w-4, STBTE__TEXTCOLOR(STBTE__cpanel));
			}
			y0 += 11;
		}
		num_rows = (y1-y0)/15;
	#ifndef STBTE_NO_PROPS
		--num_rows;
	#endif
		y = y0;
		for (i=0; i < self.num_layers; ++i) {
			char text[3], *str = (char *) self.layerinfo[i].name;
			static char lockedchar[3] = { 'U', 'P', 'L' };
			int locked = self.layerinfo[i].locked;
			int disabled = (self.solo_layer >= 0 && self.solo_layer != i);
			if (i-self.layer_scroll >= 0 && i-self.layer_scroll < num_rows) {
				if (str == NULL)
					sprintf(str=text, "%2d", i+1);
				if (stbte__button(STBTE__clayer_button, str, x0,y,(i+1<10)*2,xoff-2, STBTE__ID(STBTE__layer,i), self.cur_layer==i,0))
					self.cur_layer = (self.cur_layer == i ? -1 : i);
				if (stbte__layerbutton(x0+xoff +  0,y+1,'H',STBTE__ID(STBTE__hide,i), self.layerinfo[i].hidden,disabled,STBTE__clayer_hide))
					self.layerinfo[i].hidden = !self.layerinfo[i].hidden;
				if (stbte__layerbutton(x0+xoff + 12,y+1,lockedchar[locked],STBTE__ID(STBTE__lock,i), locked!=0,disabled,STBTE__clayer_lock))
					self.layerinfo[i].locked = (locked+1)%3;
				if (stbte__layerbutton(x0+xoff + 24,y+1,'S',STBTE__ID(STBTE__solo,i), self.solo_layer==i,0,STBTE__clayer_solo))
					self.solo_layer = (self.solo_layer == i ? -1 : i);
				y += 15;
			}
		}
		stbte__scrollbar(x1-4, y0,y-2, &self.layer_scroll, 0, self.num_layers, num_rows, STBTE__ID(STBTE__scrollbar_id, STBTE__layer));
	#ifndef STBTE_NO_PROPS
		n = stbte__text_width("prop:")+2;
		stbte__draw_text(x0,y+2, "prop:", w, STBTE__TEXTCOLOR(STBTE__cpanel));
		i = w - n - 4;
		if (i > 50) i = 50;
		if (stbte__button(STBTE__clayer_button, propmodes[self.propmode], x0+n,y,0,i, STBTE__ID(STBTE__layer,256), 0,0))
			self.propmode = (self.propmode+1)%3;
	#endif
	}

	static void stbte__categories(stbte_tilemap *tm, int x0, int y0, int w, int h)
	{
		int s=11, x,y, i;
		int num_rows = h / s;

		w -= 4;
		x = x0+2;
		y = y0+4;
		if (self.category_scroll == 0) {
			if (stbte__category_button("*ALL*", x,y, w, STBTE__ID(STBTE__categories, 65535), self.cur_category == -1)) {
				stbte__choose_category(tm, -1);
			}
			y += s;
		}

		for (i=0; i < self.num_categories; ++i) {
			if (i+1 - self.category_scroll >= 0 && i+1 - self.category_scroll < num_rows) {
				if (y + 10 > y0+h)
					return;
				if (stbte__category_button(self.categories[i], x,y,w, STBTE__ID(STBTE__categories,i), self.cur_category == i))
					stbte__choose_category(tm, i);
				y += s;
			}
		}
		stbte__scrollbar(x0+w, y0+4, y0+h-4, &self.category_scroll, 0, self.num_categories+1, num_rows, STBTE__ID(STBTE__scrollbar_id, STBTE__categories));
	}

	static void stbte__tile_in_palette(stbte_tilemap *tm, int x, int y, int slot)
	{
		stbte__tileinfo *t = &self.tiles[slot];
		int x0=x, y0=y, x1 = x+self.palette_spacing_x - 1, y1 = y+self.palette_spacing_y;
		int id = STBTE__ID(STBTE__palette, slot);
		int over = stbte__hittest(x0,y0,x1,y1, id);
		switch (stbte__ui.event) {
			case STBTE__paint:
				stbte__draw_rect(x,y,x+self.palette_spacing_x-1,y+self.palette_spacing_x-1, STBTE_COLOR_TILEPALETTE_BACKGROUND);
				STBTE_DRAW_TILE(x,y,t->id, slot == self.cur_tile,0);
				if (slot == self.cur_tile)
					stbte__draw_frame_delayed(x-1,y-1,x+self.palette_spacing_x,y+self.palette_spacing_y, STBTE_COLOR_TILEPALETTE_OUTLINE);
				break;
			default:
				if (stbte__button_core(id))
					self.cur_tile = slot;
				break;
		}
	}

	static void stbte__palette_of_tiles(stbte_tilemap *tm, int x0, int y0, int w, int h)
	{
		int i,x,y;
		int num_vis_rows = (h-6) / self.palette_spacing_y;
		int num_columns = (w-2-6) / self.palette_spacing_x;
		int num_total_rows;
		int column,row;
		int x1 = x0+w, y1=y0+h;
		x = x0+2;
		y = y0+6;

		if (num_columns == 0)
			return;

		num_total_rows = (self.cur_palette_count + num_columns-1) / num_columns; // ceil()

		column = 0;
		row	 = -self.palette_scroll;	
		for (i=0; i < self.num_tiles; ++i) {
			stbte__tileinfo *t = &self.tiles[i];

			// filter based on category
			if (self.cur_category >= 0 && t->category_id != self.cur_category)
				continue;

			// display it
			if (row >= 0 && row < num_vis_rows) {
				x = x0 + 2 + self.palette_spacing_x * column;
				y = y0 + 6 + self.palette_spacing_y * row;
				stbte__tile_in_palette(tm,x,y,i);
			}

			++column;
			if (column == num_columns) {
				column = 0;
				++row;
			}
		}
		stbte__flush_delay();
		stbte__scrollbar(x1-4, y0+6, y1-2, &self.palette_scroll, 0, num_total_rows, num_vis_rows, STBTE__ID(STBTE__scrollbar_id, STBTE__palette));
	}

	static float stbte__linear_remap(float n, float x0, float x1, float y0, float y1)
	{
		return (n-x0)/(x1-x0)*(y1-y0) + y0;
	}

	static float stbte__saved;
	static void stbte__props_panel(stbte_tilemap *tm, int x0, int y0, int w, int h)
	{
		int x1 = x0+w, y1 = y0+h;
		int i;
		int y = y0 + 5, x = x0+2;
		int slider_width = 60;
		int mx,my;
		float *p;
		short *data;
		if (!stbte__is_single_selection())
			return;
		mx = stbte__ui.select_x0;
		my = stbte__ui.select_y0;
		p = self.props[my][mx];
		data = self.data[my][mx];
		for (i=0; i < STBTE_MAX_PROPERTIES; ++i) {
			unsigned int n = STBTE_PROP_TYPE(i, data, p);
			if (n) {
				char *s = STBTE_PROP_NAME(i, data, p);
				if (s == NULL) s = "";
				switch (n & 3) {
					case STBTE_PROP_bool: {
						int flag = (int) p[i];
						if (stbte__layerbutton(x,y, flag ? 'x' : ' ', STBTE__ID(STBTE__prop_flag,i), flag, 0, 2)) {
							stbte__begin_undo(tm);
							stbte__undo_record_prop_float(tm,mx,my,i,(float) flag);
							p[i] = (float) !flag;
							stbte__end_undo(tm);
						}
						stbte__draw_text(x+13,y+1,s,x1-(x+13)-2,STBTE__TEXTCOLOR(STBTE__cpanel));
						y += 13;
						break;
					}
					case STBTE_PROP_int: {
						int a = (int) STBTE_PROP_MIN(i,data,p);
						int b = (int) STBTE_PROP_MAX(i,data,p);
						int v = (int) p[i] - a;
						if (a+v != p[i] || v < 0 || v > b-a) {
							if (v < 0) v = 0;
							if (v > b-a) v = b-a;
							p[i] = (float) (a+v); // @TODO undo
						}
						switch (stbte__slider(x, slider_width, y+7, b-a, &v, STBTE__ID(STBTE__prop_int,i)))
						{
							case STBTE__begin:
								stbte__saved = p[i];
								// fallthrough
							case STBTE__change:
								p[i] = (float) (a+v); // @TODO undo
								break;
							case STBTE__end:
								if (p[i] != stbte__saved) {
									stbte__begin_undo(tm);
									stbte__undo_record_prop_float(tm,mx,my,i,stbte__saved);
									stbte__end_undo(tm);
								}
								break;
						}
						stbte__draw_text(x+slider_width+2,y+2, s, x1-1-(x+slider_width+2), STBTE__TEXTCOLOR(STBTE__cpanel));
						y += 12;
						break;
					}
					case STBTE_PROP_float: {
						float a = (float) STBTE_PROP_MIN(i, data,p);
						float b = (float) STBTE_PROP_MAX(i, data,p);
						float c = STBTE_PROP_FLOAT_SCALE(i, data, p);
						float old;
						if (p[i] < a || p[i] > b) {
							// @TODO undo
							if (p[i] < a) p[i] = a;
							if (p[i] > b) p[i] = b;
						}
						old = p[i];
						switch (stbte__float_control(x, y, 50, a, b, c, "%8.4f", &p[i], STBTE__layer,STBTE__ID(STBTE__prop_float,i))) {
							case STBTE__begin:
								stbte__saved = old;
								break;
							case STBTE__end:
								if (stbte__saved != p[i]) {
									stbte__begin_undo(tm);
									stbte__undo_record_prop_float(tm,mx,my,i, stbte__saved);
									stbte__end_undo(tm);
								}
								break;
						}
						stbte__draw_text(x+53,y+1, s, x1-1-(x+53), STBTE__TEXTCOLOR(STBTE__cpanel));
						y += 12;
						break;
					}
				}
			}
		}
	}

	static int stbte__cp_mode, stbte__cp_aspect, stbte__cp_state, stbte__cp_index, stbte__save, stbte__cp_altered, stbte__color_copy;
	#ifdef STBTE__COLORPICKER
	static void stbte__dump_colorstate(void)
	{
		int i,j,k;
		printf("static int stbte__color_table[STBTE__num_color_modes][STBTE__num_color_aspects][STBTE__num_color_states] =\n");
		printf("{\n");
		printf("	{\n");
		for (k=0; k < STBTE__num_color_modes; ++k) {
			for (j=0; j < STBTE__num_color_aspects; ++j) {
				printf("		{ ");
				for (i=0; i < STBTE__num_color_states; ++i) {
					printf("0x%06x, ", stbte__color_table[k][j][i]);
				}
				printf("},\n");
			}
			if (k+1 < STBTE__num_color_modes)
				printf("	}, {\n");
			else
				printf("	},\n");
		}
		printf("};\n");
	}

	static void stbte__colorpicker(int x0, int y0, int w, int h)
	{
		int x1 = x0+w, y1 = y0+h, x,y, i;

		x =  x0+2; y = y0+6;

		y += 5;
		x += 8;
		
		
		{
			int color = stbte__color_table[stbte__cp_mode][stbte__cp_aspect][stbte__cp_index];
			int rgb[3];
			if (stbte__cp_altered && stbte__cp_index == STBTE__idle)
				color = stbte__save;

			if (stbte__minibutton(STBTE__cmapsize, x1-20,y+ 5, 'C', STBTE__ID2(STBTE__colorpick_id,4,0)))
				stbte__color_copy = color;
			if (stbte__minibutton(STBTE__cmapsize, x1-20,y+15, 'P', STBTE__ID2(STBTE__colorpick_id,4,1)))
				color = stbte__color_copy;

			rgb[0] = color >> 16; rgb[1] = (color>>8)&255; rgb[2] = color & 255;
			for (i=0; i < 3; ++i) {
				if (stbte__slider(x+8,64, y, 255, rgb+i, STBTE__ID2(STBTE__colorpick_id,3,i)) > 0)
					stbte__dump_colorstate();
				y += 15;
			}
			if (stbte__ui.event != STBTE__paint && stbte__ui.event != STBTE__tick)
				stbte__color_table[stbte__cp_mode][stbte__cp_aspect][stbte__cp_index] = (rgb[0]<<16)|(rgb[1]<<8)|(rgb[2]);
		}

		y += 5;

		// states
		x = x0+2+35;
		if (stbte__ui.event == STBTE__paint) {
			static char *states[] = { "idle", "over", "down", "down&over", "selected", "selected&over", "disabled" };
			stbte__draw_text(x, y+1, states[stbte__cp_index], x1-x-1, 0xffffff);
		}

		x = x0+24; y += 12;

		for (i=3; i >= 0; --i) {
			int state = 0 != (stbte__cp_state & (1 << i));
			if (stbte__layerbutton(x,y, "OASD"[i], STBTE__ID2(STBTE__colorpick_id, 0,i), state,0, STBTE__clayer_button)) {
				stbte__cp_state ^= (1 << i);
				stbte__cp_index = stbte__state_to_index[0][0][0][stbte__cp_state];
			}
			x += 16;
		}
		x = x0+2; y += 18;

		for (i=0; i < 3; ++i) {
			static char *labels[] = { "Base", "Edge", "Text" };
			if (stbte__button(STBTE__ctoolbar_button, labels[i], x,y,0,36, STBTE__ID2(STBTE__colorpick_id,1,i), stbte__cp_aspect==i,0))
				stbte__cp_aspect = i;
			x += 40;
		}

		y += 18;
		x = x0+2;

		for (i=0; i < STBTE__num_color_modes; ++i) {
			if (stbte__button(STBTE__ctoolbar_button, stbte__color_names[i], x, y, 0,80, STBTE__ID2(STBTE__colorpick_id,2,i), stbte__cp_mode == i,0))
				stbte__cp_mode = i;
			y += 12;
		}

		// make the currently selected aspect flash, unless we're actively dragging color slider etc
		if (stbte__ui.event == STBTE__tick) {
			stbte__save = stbte__color_table[stbte__cp_mode][stbte__cp_aspect][STBTE__idle];
			if ((stbte__ui.active_id & 127) != STBTE__colorpick_id) {
				if ((stbte__ui.ms_time & 2047) < 200) {
					stbte__color_table[stbte__cp_mode][stbte__cp_aspect][STBTE__idle] ^= 0x1f1f1f;
					stbte__cp_altered = 1;
				}
			}
		}
	}
	#endif

	static void stbte__editor_traverse(stbte_tilemap *tm)
	{
		int i,j,i0,j0,i1,j1,n;

		if (tm == NULL)
			return;
		if (stbte__ui.x0 == stbte__ui.x1 || stbte__ui.y0 == stbte__ui.y1)
			return;

		stbte__prepare_tileinfo(tm);

		stbte__compute_panel_locations(tm); // @OPTIMIZE: we don't need to recompute this every time

		if (stbte__ui.event == STBTE__paint) {
			// fill screen with border
			stbte__draw_rect(stbte__ui.x0, stbte__ui.y0, stbte__ui.x1, stbte__ui.y1, STBTE_COLOR_TILEMAP_BORDER);
			// fill tilemap with tilemap background
			stbte__draw_rect(stbte__ui.x0 - self.scroll_x, stbte__ui.y0 - self.scroll_y,
								stbte__ui.x0 - self.scroll_x + self.spacing_x * self.max_x,
								stbte__ui.y0 - self.scroll_y + self.spacing_y * self.max_y, STBTE_COLOR_TILEMAP_BACKGROUND);
		}

		// step 1: traverse all the tilemap data...

		i0 = (self.scroll_x - self.spacing_x) / self.spacing_x;
		j0 = (self.scroll_y - self.spacing_y) / self.spacing_y;
		i1 = (self.scroll_x + stbte__ui.x1 - stbte__ui.x0) / self.spacing_x + 1;
		j1 = (self.scroll_y + stbte__ui.y1 - stbte__ui.y0) / self.spacing_y + 1;

		if (i0 < 0) i0 = 0;
		if (j0 < 0) j0 = 0;
		if (i1 > self.max_x) i1 = self.max_x;
		if (j1 > self.max_y) j1 = self.max_y;

		if (stbte__ui.event == STBTE__paint) {
			// draw all of layer 0, then all of layer 1, etc, instead of old
			// way which drew entire stack of each tile at once
			for (n=0; n < self.num_layers; ++n) {
				for (j=j0; j < j1; ++j) {
					for (i=i0; i < i1; ++i) {
						int x = stbte__ui.x0 + i * self.spacing_x - self.scroll_x;
						int y = stbte__ui.y0 + j * self.spacing_y - self.scroll_y;
						stbte__tile_paint(tm, x, y, i, j, n);
					}
				}
				if (n == 0 && stbte__ui.show_grid == 1) {
					int x = stbte__ui.x0 + i0 * self.spacing_x - self.scroll_x;
					int y = stbte__ui.y0 + j0 * self.spacing_y - self.scroll_y;
					for (i=0; x < stbte__ui.x1 && i <= i1; ++i, x += self.spacing_x)
						stbte__draw_rect(x, stbte__ui.y0, x+1, stbte__ui.y1, STBTE_COLOR_GRID);
					for (j=0; y < stbte__ui.y1 && j <= j1; ++j, y += self.spacing_y)
						stbte__draw_rect(stbte__ui.x0, y, stbte__ui.x1, y+1, STBTE_COLOR_GRID);
				}
			}
		}

		if (stbte__ui.event == STBTE__paint) {
			// draw grid on top of everything except UI
			if (stbte__ui.show_grid == 2) {
				int x = stbte__ui.x0 + i0 * self.spacing_x - self.scroll_x;
				int y = stbte__ui.y0 + j0 * self.spacing_y - self.scroll_y;
				for (i=0; x < stbte__ui.x1 && i <= i1; ++i, x += self.spacing_x)
					stbte__draw_rect(x, stbte__ui.y0, x+1, stbte__ui.y1, STBTE_COLOR_GRID);
				for (j=0; y < stbte__ui.y1 && j <= j1; ++j, y += self.spacing_y)
					stbte__draw_rect(stbte__ui.x0, y, stbte__ui.x1, y+1, STBTE_COLOR_GRID);
			}
		}

		for (j=j0; j < j1; ++j) {
			for (i=i0; i < i1; ++i) {
				int x = stbte__ui.x0 + i * self.spacing_x - self.scroll_x;
				int y = stbte__ui.y0 + j * self.spacing_y - self.scroll_y;
				stbte__tile(tm, x, y, i, j);
			}
		}

		if (stbte__ui.event == STBTE__paint) {
			// draw the selection border
			if (stbte__ui.has_selection) {
				int x0,y0,x1,y1;
				x0 = stbte__ui.x0 + (stbte__ui.select_x0	 ) * self.spacing_x - self.scroll_x;
				y0 = stbte__ui.y0 + (stbte__ui.select_y0	 ) * self.spacing_y - self.scroll_y;
				x1 = stbte__ui.x0 + (stbte__ui.select_x1 + 1) * self.spacing_x - self.scroll_x + 1;
				y1 = stbte__ui.y0 + (stbte__ui.select_y1 + 1) * self.spacing_y - self.scroll_y + 1;
				stbte__draw_frame(x0,y0,x1,y1, (stbte__ui.ms_time & 256 ? STBTE_COLOR_SELECTION_OUTLINE1 : STBTE_COLOR_SELECTION_OUTLINE2));
			}

			stbte__flush_delay(); // draw a dynamic link on top of the queued links

			#ifdef STBTE_ALLOW_LINK
			if (stbte__ui.linking && STBTE__IS_MAP_HOT()) {
				int x0,y0,x1,y1;
				int color;
				int ex = ((stbte__ui.hot_id >> 19) & 4095);
				int ey = ((stbte__ui.hot_id >>  7) & 4095);
				x0 = stbte__ui.x0 + (stbte__ui.sx	 ) * self.spacing_x - self.scroll_x + (self.spacing_x>>1)+1;
				y0 = stbte__ui.y0 + (stbte__ui.sy	 ) * self.spacing_y - self.scroll_y + (self.spacing_y>>1)+1;
				x1 = stbte__ui.x0 + (ex				  ) * self.spacing_x - self.scroll_x + (self.spacing_x>>1)-1;
				y1 = stbte__ui.y0 + (ey				  ) * self.spacing_y - self.scroll_y + (self.spacing_y>>1)-1;
				if (STBTE_ALLOW_LINK(self.data[stbte__ui.sy][stbte__ui.sx], self.props[stbte__ui.sy][stbte__ui.sx], self.data[ey][ex], self.props[ey][ex]))
					color = STBTE_LINK_COLOR_DRAWING;
				else
					color = STBTE_LINK_COLOR_DISALLOWED;
				stbte__draw_link(x0,y0,x1,y1, color);
			}
			#endif
		}
		stbte__flush_delay();

		// step 2: traverse the panels
		for (i=0; i < STBTE__num_panel; ++i) {
			stbte__panel *p = &stbte__ui.panel[i];
			if (stbte__ui.event == STBTE__paint) {
				stbte__draw_box(p->x0,p->y0,p->x0+p->width,p->y0+p->height, STBTE__cpanel, STBTE__idle);
			}
			// obscure tilemap data underneath panel
			stbte__hittest(p->x0,p->y0,p->x0+p->width,p->y0+p->height, STBTE__ID2(STBTE__panel, i, 0));
			switch (i) {
				case STBTE__panel_toolbar:
					if (stbte__ui.event == STBTE__paint)
						stbte__draw_rect(p->x0,p->y0,p->x0+p->width,p->y0+p->height, stbte__color_table[STBTE__ctoolbar][STBTE__base][STBTE__idle]);
					stbte__toolbar(tm,p->x0,p->y0,p->width,p->height);
					break;
				case STBTE__panel_info:
					stbte__info(tm,p->x0,p->y0,p->width,p->height);
					break;
				case STBTE__panel_layers:
					stbte__layers(tm,p->x0,p->y0,p->width,p->height);
					break;
				case STBTE__panel_categories:
					stbte__categories(tm,p->x0,p->y0,p->width,p->height);
					break;
				case STBTE__panel_colorpick:
	#ifdef STBTE__COLORPICKER
					stbte__colorpicker(p->x0,p->y0,p->width,p->height);
	#endif
					break;
				case STBTE__panel_tiles:
					// erase boundary between categories and tiles if they're on same side
					if (stbte__ui.event == STBTE__paint && p->side == stbte__ui.panel[STBTE__panel_categories].side)
						stbte__draw_rect(p->x0+1,p->y0-1,p->x0+p->width-1,p->y0+1, stbte__color_table[STBTE__cpanel][STBTE__base][STBTE__idle]);
					stbte__palette_of_tiles(tm,p->x0,p->y0,p->width,p->height);
					break;
				case STBTE__panel_props:
					stbte__props_panel(tm,p->x0,p->y0,p->width,p->height);
					break;
			}
			// draw the panel side selectors
			for (j=0; j < 2; ++j) {
				int result;
				if (i == STBTE__panel_toolbar) continue;
				result = stbte__microbutton(p->x0+p->width - 1 - 2*4 + 4*j,p->y0+2,3, STBTE__ID2(STBTE__panel, i, j+1), STBTE__cpanel_sider+j);
				if (result) {
					switch (j) {
						case 0: p->side = result > 0 ? STBTE__side_left : STBTE__side_right; break;
						case 1: p->delta_height += result; break;
					}
				}
			}
		}

		if (stbte__ui.panel[STBTE__panel_categories].delta_height < -5) stbte__ui.panel[STBTE__panel_categories].delta_height = -5;
		if (stbte__ui.panel[STBTE__panel_layers	 ].delta_height < -5) stbte__ui.panel[STBTE__panel_layers	 ].delta_height = -5;


		// step 3: traverse the regions to place expander controls on them
		for (i=0; i < 2; ++i) {
			if (stbte__region[i].active) {
				int x = stbte__region[i].x;
				int width;
				if (i == STBTE__side_left)
					width =  stbte__ui.left_width , x += stbte__region[i].width + 1;
				else
					width = -stbte__ui.right_width, x -= 6;
				if (stbte__microbutton_dragger(x, stbte__region[i].y+2, 5, STBTE__ID(STBTE__region,i), &width)) {
					// if non-0, it is expanding, so retract it
					if (stbte__region[i].retracted == 0.0)
						stbte__region[i].retracted = 0.01f;
					else
						stbte__region[i].retracted = 0.0;
				}
				if (i == STBTE__side_left)
					stbte__ui.left_width  =  width;
				else
					stbte__ui.right_width = -width;
				if (stbte__ui.event == STBTE__tick) {
					if (stbte__region[i].retracted && stbte__region[i].retracted < 1.0f) {
						stbte__region[i].retracted += stbte__ui.dt*4;
						if (stbte__region[i].retracted > 1)
							stbte__region[i].retracted = 1;
					}
				}
			}
		}

		if (stbte__ui.event == STBTE__paint && stbte__ui.alert_msg) {
			int w = stbte__text_width(stbte__ui.alert_msg);
			int x = (stbte__ui.x0+stbte__ui.x1)/2;
			int y = (stbte__ui.y0+stbte__ui.y1)*5/6;
			stbte__draw_rect (x-w/2-4,y-8, x+w/2+4,y+8, 0x604020);
			stbte__draw_frame(x-w/2-4,y-8, x+w/2+4,y+8, 0x906030);
			stbte__draw_text (x-w/2,y-4, stbte__ui.alert_msg, w+1, 0xff8040);
		}

	#ifdef STBTE_SHOW_CURSOR
		if (stbte__ui.event == STBTE__paint)
			stbte__draw_bitmap(stbte__ui.mx, stbte__ui.my, stbte__get_char_width(26), stbte__get_char_bitmap(26), 0xe0e0e0);
	#endif

		if (stbte__ui.event == STBTE__tick && stbte__ui.alert_msg) {
			stbte__ui.alert_timer -= stbte__ui.dt;
			if (stbte__ui.alert_timer < 0) {
				stbte__ui.alert_timer = 0;
				stbte__ui.alert_msg = 0;
			}
		}

		if (stbte__ui.event == STBTE__paint) {
			stbte__color_table[stbte__cp_mode][stbte__cp_aspect][STBTE__idle] = stbte__save;
			stbte__cp_altered = 0;
		}
	}

	static void stbte__do_event(stbte_tilemap *tm)
	{
		stbte__ui.next_hot_id = 0;
		stbte__editor_traverse(tm);
		stbte__ui.hot_id = stbte__ui.next_hot_id;

		// automatically cancel on mouse-up in case the object that triggered it
		// doesn't exist anymore
		if (stbte__ui.active_id) {
			if (stbte__ui.event == STBTE__leftup || stbte__ui.event == STBTE__rightup) {
				if (!stbte__ui.pasting) {
					stbte__activate(0);
					if (stbte__ui.undoing)
						stbte__end_undo(tm);
					stbte__ui.scrolling = 0;
					stbte__ui.dragging = 0;
					stbte__ui.linking = 0;
				}
			}
		}

		// we could do this stuff in the widgets directly, but it would keep recomputing
		// the same thing on every tile, which seems dumb.

		if (stbte__ui.pasting) {
			if (STBTE__IS_MAP_HOT()) {
				// compute pasting location based on last hot
				stbte__ui.paste_x = ((stbte__ui.hot_id >> 19) & 4095) - (stbte__ui.copy_width >> 1);
				stbte__ui.paste_y = ((stbte__ui.hot_id >>  7) & 4095) - (stbte__ui.copy_height >> 1);
			}
		}
		if (stbte__ui.dragging) {
			if (STBTE__IS_MAP_HOT()) {
				stbte__ui.drag_dest_x = ((stbte__ui.hot_id >> 19) & 4095) - stbte__ui.drag_offx;
				stbte__ui.drag_dest_y = ((stbte__ui.hot_id >>  7) & 4095) - stbte__ui.drag_offy;
			}
		}
	}

	static void stbte__set_event(int event, int x, int y)
	{
		stbte__ui.event = event;
		stbte__ui.mx	 = x;
		stbte__ui.my	 = y;
		stbte__ui.dx	 = x - stbte__ui.last_mouse_x;
		stbte__ui.dy	 = y - stbte__ui.last_mouse_y;
		stbte__ui.last_mouse_x = x;
		stbte__ui.last_mouse_y = y;
		stbte__ui.accum_x += stbte__ui.dx;
		stbte__ui.accum_y += stbte__ui.dy;
	}

	void stbte_draw(stbte_tilemap *tm)
	{
		stbte__ui.event = STBTE__paint;
		stbte__editor_traverse(tm);
	}

	void stbte_mouse_move(stbte_tilemap *tm, int x, int y, int shifted, int scrollkey)
	{
		stbte__set_event(STBTE__mousemove, x,y);
		stbte__ui.shift = shifted;
		stbte__ui.scrollkey = scrollkey;
		stbte__do_event(tm);
	}

	void stbte_mouse_button(stbte_tilemap *tm, int x, int y, int right, int down, int shifted, int scrollkey)
	{
		static int events[2][2] = { { STBTE__leftup , STBTE__leftdown  },
											{ STBTE__rightup, STBTE__rightdown } };
		stbte__set_event(events[right][down], x,y);
		stbte__ui.shift = shifted;
		stbte__ui.scrollkey = scrollkey;

		stbte__do_event(tm);
	}

	void stbte_mouse_wheel(stbte_tilemap *tm, int x, int y, int vscroll)
	{
		// not implemented yet -- need different way of hittesting
	}

	void stbte_action(stbte_tilemap *tm, enum stbte_action act)
	{
		switch (act) {
			case STBTE_tool_select:		stbte__ui.tool = STBTE__tool_select;					break;
			case STBTE_tool_brush:		 stbte__ui.tool = STBTE__tool_brush;					 break;
			case STBTE_tool_erase:		 stbte__ui.tool = STBTE__tool_erase;					 break;
			case STBTE_tool_rectangle:	stbte__ui.tool = STBTE__tool_rect;					  break;
			case STBTE_tool_eyedropper:  stbte__ui.tool = STBTE__tool_eyedrop;				  break;
			case STBTE_tool_link:		  stbte__ui.tool = STBTE__tool_link;					  break;
			case STBTE_act_toggle_grid:  stbte__ui.show_grid = (stbte__ui.show_grid+1) % 3; break;
			case STBTE_act_toggle_links: stbte__ui.show_links ^= 1;								 break;
			case STBTE_act_undo:			stbte__undo(tm);											  break;
			case STBTE_act_redo:			stbte__redo(tm);											  break;
			case STBTE_act_cut:			 stbte__copy_cut(tm, 1);									 break;
			case STBTE_act_copy:			stbte__copy_cut(tm, 0);									 break;
			case STBTE_act_paste:		  stbte__start_paste(tm);									 break;
			case STBTE_scroll_left:		self.scroll_x -= self.spacing_x;							break;
			case STBTE_scroll_right:	  self.scroll_x += self.spacing_x;							break;
			case STBTE_scroll_up:		  self.scroll_y -= self.spacing_y;							break;
			case STBTE_scroll_down:		self.scroll_y += self.spacing_y;							break;
		}
	}

	void stbte_tick(stbte_tilemap *tm, float dt)
	{
		stbte__ui.event = STBTE__tick;
		stbte__ui.dt	 = dt;
		stbte__do_event(tm);
		stbte__ui.ms_time += (int) (dt * 1024) + 1; // make sure if time is superfast it always updates a little
	}

	void stbte_mouse_sdl(stbte_tilemap *tm, const void *sdl_event, float xs, float ys, int xo, int yo)
	{
	#ifdef _SDL_H
		SDL_Event *event = (SDL_Event *) sdl_event;
		SDL_Keymod km = SDL_GetModState();
		int shift = (km & KMOD_LCTRL) || (km & KMOD_RCTRL);
		int scrollkey = 0 != SDL_GetKeyboardState(NULL)[SDL_SCANCODE_SPACE];
		switch (event->type) {
			case SDL_MOUSEMOTION:
				stbte_mouse_move(tm, (int) (xs*event->motion.x+xo), (int) (ys*event->motion.y+yo), shift, scrollkey);
				break;
			case SDL_MOUSEBUTTONUP:
				stbte_mouse_button(tm, (int) (xs*event->button.x+xo), (int) (ys*event->button.y+yo), event->button.button != SDL_BUTTON_LEFT, 0, shift, scrollkey);
				break;
			case SDL_MOUSEBUTTONDOWN:
				stbte_mouse_button(tm, (int) (xs*event->button.x+xo), (int) (ys*event->button.y+yo), event->button.button != SDL_BUTTON_LEFT, 1, shift, scrollkey);
				break;
			case SDL_MOUSEWHEEL:
				stbte_mouse_wheel(tm, stbte__ui.mx, stbte__ui.my, event->wheel.y);
				break;
		}
	#else
		STBTE__NOTUSED(tm);
		STBTE__NOTUSED(sdl_event);
		STBTE__NOTUSED(xs);
		STBTE__NOTUSED(ys);
		STBTE__NOTUSED(xo);
		STBTE__NOTUSED(yo);
	#endif
	}

	#endif // STB_TILEMAP_EDITOR_IMPLEMENTATION
	*/