enum Corner {
	TopRight,
	BottomRight,
	BottomLeft,
	TopLeft,
}

enum Edge {
	Top,
	Right,
	Bottom,
	Left,
}

struct Id(u32);

struct IdVariation {
	id: Id,
	edge_color: isize,
	corner_color: isize,
}

struct Tile<T> {
	tile: Rc<T>,
	id: Id,
	flipped_horizontally: bool,
	flipped_vertically: bool,
	flipped_anti_diagonally: bool,
}

struct Color<C> {
	color_index: isize,
	is_edge: bool,
	name: String,
	image_id: isize,
	probability: f32,
	color: C;
}

struct Tileset;

struct Set {
	ts: Rc<Tileset>,
	name: String,
	image_tile_id: isize,
	edge_colors: Vec<Color>,
	corner_colors: Vec<Color>,
	unique_full_wang_id_count: u32;
}