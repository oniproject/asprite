


///////////////////////////////////////////////////////////////////////
//
//	HEADER SECTION

/*
#ifndef STB_TILEMAP_INCLUDE_STB_TILEMAP_EDITOR_H
#define STB_TILEMAP_INCLUDE_STB_TILEMAP_EDITOR_H

#ifdef _WIN32
  #ifndef _CRT_SECURE_NO_WARNINGS
  #define _CRT_SECURE_NO_WARNINGS
  #endif
  #include <stdlib.h>
  #include <stdio.h>
#endif

typedef struct stbte_tilemap stbte_tilemap;
*/

// these are the drawmodes used in STBTE_DRAW_TILE
enum DrawMode {
	Deemphasize = -1,
	Normal		=  0,
	Emphasize	=  1,
}

/*
// these are the property types
#define STBTE_PROP_none	  0
#define STBTE_PROP_int		1
#define STBTE_PROP_float	 2
#define STBTE_PROP_bool	  3
#define STBTE_PROP_disabled 4

////////
//
// creation
//

extern stbte_tilemap *stbte_create_map(int map_x, int map_y, int map_layers, int spacing_x, int spacing_y, int max_tiles);
// create an editable tilemap
//	map_x		: dimensions of map horizontally (user can change this in editor), <= STBTE_MAX_TILEMAP_X
//	map_y		: dimensions of map vertically (user can change this in editor)	 <= STBTE_MAX_TILEMAP_Y
//	map_layers : number of layers to use (fixed), <= STBTE_MAX_LAYERS
//	spacing_x  : initial horizontal distance between left edges of map tiles in stb_tilemap_editor pixels
//	spacing_y  : initial vertical distance between top edges of map tiles in stb_tilemap_editor pixels
//	max_tiles  : maximum number of tiles that can defined
//
// If insufficient memory, returns NULL

extern void stbte_define_tile(stbte_tilemap *tm, unsigned short id, unsigned int layermask, const char * category);
// call this repeatedly for each tile to install the tile definitions into the editable tilemap
//	tm		  : tilemap created by stbte_create_map
//	id		  : unique identifier for each tile, 0 <= id < 32768
//	layermask : bitmask of which layers tile is allowed on: 1 = layer 0, 255 = layers 0..7
//					(note that onscreen, the editor numbers the layers from 1 not 0)
//					layer 0 is the furthest back, layer 1 is just in front of layer 0, etc
//	category  : which category this tile is grouped in

extern void stbte_set_display(int x0, int y0, int x1, int y1);
// call this once to set the size; if you resize, call it again


/////////
//
// every frame
//

extern void stbte_draw(stbte_tilemap *tm);

extern void stbte_tick(stbte_tilemap *tm, float time_in_seconds_since_last_frame);

////////////
//
//  user input
//

// if you're using SDL, call the next function for SDL_MOUSEMOVE, SDL_MOUSEBUTTON, SDL_MOUSEWHEEL;
// the transformation lets you scale from SDL mouse coords to stb_tilemap_editor coords
extern void stbte_mouse_sdl(stbte_tilemap *tm, const void *sdl_event, float xscale, float yscale, int xoffset, int yoffset);

// otherwise, hook these up explicitly:
extern void stbte_mouse_move(stbte_tilemap *tm, int x, int y, int shifted, int scrollkey);
extern void stbte_mouse_button(stbte_tilemap *tm, int x, int y, int right, int down, int shifted, int scrollkey);
extern void stbte_mouse_wheel(stbte_tilemap *tm, int x, int y, int vscroll);
*/

// for keyboard, define your own mapping from keys to the following actions.
// this is totally optional, as all features are accessible with the mouse
enum Action {
	ToolSelect,
	ToolBrush,
	ToolErase,
	ToolRectangle,
	ToolEyedropper,
	ToolLink,

	ActToggleGrid,
	ActToggleLinks,
	ActUndo,
	ActRedo,
	ActCut,
	ActCopy,
	ActPaste,

	ScrollLeft,
	ScrollRight,
	ScrollUp,
	ScrollDown,
}
/*
extern void stbte_action(stbte_tilemap *tm, enum stbte_action act);

////////////////
//
//  save/load 
//
//  There is no editor file format. You have to save and load the data yourself
//  through the following functions. You can also use these functions to get the
//  data to generate game-formatted levels directly. (But make sure you save
//  first! You may also want to autosave to a temp file periodically, etc etc.)

#define STBTE_EMPTY	 -1

extern void stbte_get_dimensions(stbte_tilemap *tm, int *max_x, int *max_y);
// get the dimensions of the level, since the user can change them

extern short* stbte_get_tile(stbte_tilemap *tm, int x, int y);
// returns an array of shorts that is 'map_layers' in length. each short is
// either one of the tile_id values from define_tile, or STBTE_EMPTY.

extern float *stbte_get_properties(stbte_tilemap *tm, int x, int y);
// get the property array associated with the tile at x,y. this is an
// array of floats that is STBTE_MAX_PROPERTIES in length; you have to
// interpret the slots according to the semantics you've chosen

extern void stbte_get_link(stbte_tilemap *tm, int x, int y, int *destx, int *desty);
// gets the link associated with the tile at x,y.

extern void stbte_set_dimensions(stbte_tilemap *tm, int max_x, int max_y);
// set the dimensions of the level, overrides previous stbte_create_map()
// values or anything the user has changed

extern void stbte_clear_map(stbte_tilemap *tm);
// clears the map, including the region outside the defined region, so if the
// user expands the map, they won't see garbage there

extern void stbte_set_tile(stbte_tilemap *tm, int x, int y, int layer, signed short tile);
// tile is your tile_id from define_tile, or STBTE_EMPTY

extern void stbte_set_property(stbte_tilemap *tm, int x, int y, int n, float val);
// set the value of the n'th slot of the tile at x,y

extern void stbte_set_link(stbte_tilemap *tm, int x, int y, int destx, int desty);
// set a link going from x,y to destx,desty. to force no link,
// use destx=desty=-1

////////
//
// optional
//

extern void stbte_set_background_tile(stbte_tilemap *tm, short id);
// selects the tile to fill the bottom layer with and used to clear bottom tiles to;
// should be same ID as 

extern void stbte_set_sidewidths(int left, int right);
// call this once to set the left & right side widths. don't call
// it again since the user can change it

extern void stbte_set_spacing(stbte_tilemap *tm, int spacing_x, int spacing_y, int palette_spacing_x, int palette_spacing_y);
// call this to set the spacing of map tiles and the spacing of palette tiles.
// if you rescale your display, call it again (e.g. you can implement map zooming yourself)

extern void stbte_set_layername(stbte_tilemap *tm, int layer, const char *layername);
// sets a string name for your layer that shows in the layer selector. note that this
// makes the layer selector wider. 'layer' is from 0..(map_layers-1)

#endif
*/


