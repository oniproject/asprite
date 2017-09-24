const FLIPPED_HORIZONTALLY: u32 = 0x80000000;
const FLIPPED_VERTICALLY: u32   = 0x40000000;
const FLIPPED_DIAGONALLY: u32   = 0x20000000;

const IDX_MASK: u32 = !(FLIPPED_HORIZONTALLY | FLIPPED_VERTICALLY | FLIPPED_DIAGONALLY);

struct GlobalTileId(u32);

impl GlobalTileId {
	fn new(id: u32) -> Self {
		TileId(id)
	}
	pub fn idx(self) -> usize {
		(self.0 & IDX_MASK) as usize
	}

	pub fn flipped_horizontally(&self) -> bool {
		self.0 & FLIPPED_HORIZONTALLY != 0
	}
	pub fn flipped_vertically(&self) -> bool {
		self.0 & FLIPPED_VERTICALLY != 0
	}
	pub fn flipped_diagonally(&self) -> bool {
		self.0 & FLIPPED_DIAGONALLY != 0
	}
}

enum Orientation {
	Orthogonal,
	Isometric,
	Staggered,
	Hexagonal,
}

enum RenderOrder {
	RightDown, // default
	RightUp,
	LeftDown,
	LeftUp,
}

struct Map {
	orientation: Orientation,
	renderorder: RenderOrder,
	width: usize,
	height: usize,
	tile_width: usize,
	tile_height: usize,
	hex_side_length: usize,
	stage_axis: usize,
	bg_rgba: u32,
	next_object_id: usize,

	// <properties>, <tileset>, <layer>, <objectgroup>, <imagelayer>, <group> (since 1.0), <templategroup>
}

struct TileSet {
/*
    firstgid: The first global tile ID of this tileset (this global ID maps to the first tile in this tileset).
    source: If this tileset is stored in an external TSX (Tile Set XML) file, this attribute refers to that file. That TSX file has the same structure as the <tileset> element described here. (There is the firstgid attribute missing and this source attribute is also not there. These two attributes are kept in the TMX map, since they are map specific.)
    name: The name of this tileset.
    tile_width: The (maximum) width of the tiles in this tileset.
    tile_height: The (maximum) height of the tiles in this tileset.
    spacing: The spacing in pixels between the tiles in this tileset (applies to the tileset image).
    margin: The margin around the tiles in this tileset (applies to the tileset image).
    tilecount: The number of tiles in this tileset (since 0.13)
    columns: The number of tile columns in the tileset. For image collection tilesets it is editable and is used when displaying the tileset. (since 0.15)
*/

// <tileoffset>, <properties>, <image>, <terraintypes>, <tile>, <wangsets> (since 1.1)

}

struct TileOffset {
	x: isize,
	y: isize,
}

type Terrain = HashMap<String, TileId>;

type TileId = usize;

struct Tile {
	id: LocalTileId,

/*
id: The local tile ID within its tileset.
type: The type of the tile. Refers to an object type and is used by tile objects. (optional) (since 1.0)
terrain: Defines the terrain type of each corner of the tile, given as comma-separated indexes in the terrain types array in the order top-left, top-right, bottom-left, bottom-right. Leaving out a value means that corner has no terrain. (optional)
probability: A percentage indicating the probability that this tile is chosen when it competes with others while editing with the terrain tool. (optional)

Can contain: <properties>, <image> (since 0.9), <objectgroup>, <animation>
*/
}

type Animation = Vec<Frame>;

struct Frame {
	tile: TileId,
	duration: u16,
}

type WangSets = HashMap<String, Wang>;

struct Wang {
	tile: TileId,
}
