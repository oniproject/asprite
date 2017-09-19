
// stb_tilemap_editor.h - v0.38 - Sean Barrett - http://nothings.org/stb
// placed in the public domain - not copyrighted - first released 2014-09
//
// Embeddable tilemap editor for C/C++
//
//
// TABLE OF CONTENTS
//	 FAQ
//	 How to compile/use the library
//	 Additional configuration macros
//	 API documentation
//	 Info on editing multiple levels
//	 Revision history
//	 Todo
//	 Credits
//	 License
//
//
// FAQ
//
//	Q: What counts as a tilemap for this library?
//
//	A: An array of rectangles, where each rectangle contains a small
//		stack of images.
//
//	Q: What are the limitations?
//
//	A: Maps are limited to 4096x4096 in dimension.
//		Each map square can only contain a stack of at most 32 images.
//		A map can only use up to 32768 distinct image tiles.
//
//	Q: How do I compile this?
//
//	A: You need to #define several symbols before #including it, but only
//		in one file. This will cause all the function definitions to be
//		generated in that file. See the "HOW TO COMPILE" section.
//
//	Q: What advantages does this have over a standalone editor?
//
//	A: For one, you can integrate the editor into your game so you can
//		flip between editing and testing without even switching windows.
//		For another, you don't need an XML parser to get at the map data.
//
//	Q: Can I live-edit my game maps?
//
//	A: Not really, the editor keeps its own map representation.
//
//	Q: How do I save and load maps?
//
//	A: You have to do this yourself. The editor provides serialization
//		functions (get & set) for reading and writing the map it holds.
//		You can choose whatever format you want to store the map to on
//		disk; you just need to provide functions to convert. (For example,
//		I actually store the editor's map representation to disk basically
//		as-is; then I have a single function that converts from the editor
//		map representation to the game representation, which is used both
//		to go from editor-to-game and from loaded-map-to-game.)
//
//	Q: I want to have tiles change appearance based on what's
//		adjacent, or other tile-display/substitution trickiness.
//
//	A: You can do this when you convert from the editor's map
//		representation to the game representation, but there's
//		no way to show this live in the editor.
//
//	Q: The editor appears to be put map location (0,0) at the top left?
//		I want to use a different coordinate system in my game (e.g. y
//		increasing upwards, or origin at the center).
//
//	A: You can do this when you convert from the editor's map
//		representation to the game representation. (Don't forget to
//		translate link coordinates as well!)
//
//	Q: The editor appears to put pixel (0,0) at the top left? I want
//		to use a different coordinate system in my game.
//
//	A: The editor defines an "editor pixel coordinate system" with
//		(0,0) at the top left and requires you to display things in
//		that coordinate system. You can freely remap those coordinates
//		to anything you want on screen.
//
//	Q: How do I scale the user interface?
//
//	A: Since you do all the rendering, you can scale up all the rendering
//		calls that the library makes to you. If you do, (a) you need
//		to also scale up the mouse coordinates, and (b) you may want
//		to scale the map display back down so that you're only scaling
//		the UI and not everything. See the next question.
//
//	Q: How do I scale the map display?
//
//	A: Use stbte_set_spacing() to change the size that the map is displayed
//		at. Note that the "callbacks" to draw tiles are used for both drawing
//		the map and drawing the tile palette, so that callback may need to
//		draw at two different scales. You should choose the scales to match
//		 You can tell them apart because the
//		tile palette gets NULL for the property pointer.
//
//	Q: How does object editing work?
//
//	A: One way to think of this is that in the editor, you're placing
//		spawners, not objects. Each spawner must be tile-aligned, because
//		it's only a tile editor. Each tile (stack of layers) gets
//		an associated set of properties, and it's up to you to
//		determine what properties should appear for a given tile,
//		based on e.g. the spawners that are in it.
//
//	Q: How are properties themselves handled?
//
//	A: All properties, regardless of UI behavior, are internally floats.
//		Each tile has an array of floats associated with it, which is
//		passed back to you when drawing the tiles so you can draw
//		objects appropriately modified by the properties.
//
//	Q: What if I want to have two different objects/spawners in
//		one tile, both of which have their own properties?
//
//	A: Make sure STBTE_MAX_PROPERTIES is large enough for the sum of
//		properties in both objects, and then you have to explicitly
//		map the property slot #s to the appropriate objects. They'll
//		still all appear in a single property panel; there's no way
//		to get multiple panels.
//
//	Q: Can I do one-to-many linking?
//
//	A: The library only supports one link per tile. However, you
//		can have multiple tiles all link to a single tile. So, you
//		can fake one-to-many linking by linking in the reverse
//		direction.
//
//	Q: What if I have two objects in the same tile, and they each
//		need an independent link? Or I have two kinds of link associated
//		with a single object?
//
//	A: There is no way to do this. (Unless you can reverse one link.)
//
//	Q: How does cut & paste interact with object properties & links?
//
//	A: Currently the library has no idea which properties or links
//		are associated with which layers of a tile. So currently, the
//		library will only copy properties & links if the layer panel
//		is set to allow all layers to be copied, OR if you set the
//		"props" in the layer panel to "always". Similarly, you can
//		set "props" to "none" so it will never copy.
//
//	Q: What happens if the library gets a memory allocation failure
//		while I'm editing? Will I lose my work?
//
//	A: The library allocates all editor memory when you create
//		the tilemap. It allocates a maximally-sized map and a 
//		fixed-size undo buffer (and the fixed-size copy buffer
//		is static), and never allocates memory while it's running.
//		So it can't fail due to running out of memory.
//
//	Q: What happens if the library crashes while I'm editing? Will
//		I lose my work?
//
//	A: Yes. Save often.
//
//
// HOW TO COMPILE
//
//	This header file contains both the header file and the
//	implementation file in one. To create the implementation,
//	in one source file define a few symbols first and then
//	include this header:
//
//		#define STB_TILEMAP_EDITOR_IMPLEMENTATION
//		// this triggers the implementation
//
//		void STBTE_DRAW_RECT(int x0, int y0, int x1, int y1, uint color);
//		// this must draw a filled rectangle (exclusive on right/bottom)
//		// color = (r<<16)|(g<<8)|(b)
//		
//		void STBTE_DRAW_TILE(int x0, int y0,
//						  unsigned short id, int highlight, float *data);
//		// this draws the tile image identified by 'id' in one of several
//		// highlight modes (see STBTE_drawmode_* in the header section);
//		// if 'data' is NULL, it's drawing the tile in the palette; if 'data'
//		// is not NULL, it's drawing a tile on the map, and that is the data
//		// associated with that map tile
//
//		#include "stb_tilemap_editor.h"
//
//	Optionally you can define the following functions before the include;
//	note these must be macros (but they can just call a function) so
//	this library can #ifdef to detect if you've defined them:
//
//		#define STBTE_PROP_TYPE(int n, short *tiledata, float *params) ...
//		// Returns the type of the n'th property of a given tile, which
//		// controls how it is edited. Legal types are:
//		//	  0						  /* no editable property in this slot */
//		//	  STBTE_PROP_int		 /* uses a slider to adjust value	  */
//		//	  STBTE_PROP_float	  /* uses a weird multi-axis control	*/
//		//	  STBTE_PROP_bool		/* uses a checkbox to change value	*/
//		// And you can bitwise-OR in the following flags:
//		//	  STBTE_PROP_disabled
//		// Note that all of these are stored as floats in the param array.
//		// The integer slider is limited in precision based on the space
//		// available on screen, so for wide-ranged integers you may want
//		// to use floats instead.
//		//
//		// Since the tiledata is passed to you, you can choose which property
//		// is bound to that slot based on that data.
//		//
//		// Changing the type of a parameter does not cause the underlying
//		// value to be clamped to the type min/max except when the tile is
//		// explicitly selected.
// 
//		#define STBTE_PROP_NAME(int n, short *tiledata, float *params) ...
//		// these return a string with the name for slot #n in the float
//		// property list for the tile.
//
//		#define STBTE_PROP_MIN(int n, short *tiledata) ...your code here...
//		#define STBTE_PROP_MAX(int n, short *tiledata) ...your code here...
//		// These return the allowable range for the property values for
//		// the specified slot. It is never called for boolean types.
//
//		#define STBTE_PROP_FLOAT_SCALE(int n, short *tiledata, float *params)
//		// This rescales the float control for a given property; by default
//		// left mouse drags add integers, right mouse drags adds fractions,
//		// but you can rescale this per-property.
//
//		#define STBTE_FLOAT_CONTROL_GRANULARITY		 ... value ...
//		// This returns the number of pixels of mouse motion necessary
//		// to advance the object float control. Default is 4
//
//		#define STBTE_ALLOW_LINK(short *src, float *src_data,  \
//										 short *dest, float *dest_data) ...your code...
//		// this returns true or false depending on whether you allow a link
//		// to be drawn from a tile 'src' to a tile 'dest'. if you don't
//		// define this, linking will not be supported
//
//		#define STBTE_LINK_COLOR(short *src, float *src_data,  \
//										 short *dest, float *dest_data) ...your code...
//		// return a color encoded as a 24-bit unsigned integer in the
//		// form 0xRRGGBB. If you don't define this, default colors will
//		// be used.
//
//
//		[[ support for those below is not implemented yet ]]
//
//		#define STBTE_HITTEST_TILE(x0,y0,id,mx,my)	...your code here...
//		// this returns true or false depending on whether the mouse
//		// pointer at mx,my is over (touching) a tile of type 'id'
//		// displayed at x0,y0. Normally stb_tilemap_editor just does
//		// this hittest based on the tile geometry, but if you have
//		// tiles whose images extend out of the tile, you'll need this.
//
// ADDITIONAL CONFIGURATION
//
//	The following symbols set static limits which determine how much
//	memory will be allocated for the editor. You can override them
//	by making similiar definitions, but memory usage will increase.
//
//		#define STBTE_MAX_TILEMAP_X		200	// max 4096
//		#define STBTE_MAX_TILEMAP_Y		200	// max 4096
//		#define STBTE_MAX_LAYERS			8	  // max 32
//		#define STBTE_MAX_CATEGORIES	  100
//		#define STBTE_UNDO_BUFFER_BYTES  (1 << 24) // 16 MB
//		#define STBTE_MAX_COPY			  90000  // e.g. 300x300
//		#define STBTE_MAX_PROPERTIES	  10	  // max properties per tile
//
// API
//
//	Further documentation appears in the header-file section below.
//
// EDITING MULTIPLE LEVELS
//
//	You can only have one active editor instance. To switch between multiple
//	levels, you can either store the levels in your own format and copy them
//	in and out of the editor format, or you can create multiple stbte_tilemap
//	objects and switch between them. The latter has the advantage that each
//	stbte_tilemap keeps its own undo state. (The clipboard is global, so
//	either approach allows cut&pasting between levels.)
//
// REVISION HISTORY
//	0.38  fix warning
//	0.37  fix warning
//	0.36  minor compiler support
//	0.35  layername button changes
//			 - layername buttons grow with the layer panel
//			 - fix stbte_create_map being declared as stbte_create
//			 - fix declaration of stbte_create_map
//	0.30  properties release
//			 - properties panel for editing user-defined "object" properties
//			 - can link each tile to one other tile
//			 - keyboard interface
//			 - fix eraser tool bug (worked in complex cases, failed in simple)
//			 - undo/redo tools have visible disabled state
//			 - tiles on higher layers draw on top of adjacent lower-layer tiles
//	0.20  erasable release
//			 - eraser tool
//			 - fix bug when pasting into protected layer
//			 - better color scheme
//			 - internal-use color picker
//	0.10  initial release 
//
// TODO
//
//	Separate scroll state for each category
//	Implement paint bucket
//	Support STBTE_HITTEST_TILE above
//  ?Cancel drags by clicking other button? - may be fixed
//	Finish support for toolbar at side
//
// CREDITS
//
//
//	Main editor & features
//		Sean Barrett
//	Additional features:
//		Josh Huelsman
//	Bugfixes:
//		Ryan Whitworth
//		Eugene Opalev
//
// LICENSE
//
//	See end of file for license information.


