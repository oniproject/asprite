pub struct Map {
	data: Vec<Vec<>>,
}

pub struct Tileset {}

bitflags! {
    struct TileFlags: u32 {
		const FLIPPED_HORIZONTALLY    = 0b00000001;
		const FLIPPED_VERTICALLY      = 0b00000010;
		const FLIPPED_ANTI_DIAGONALLY = 0b00000100;
		const ROTATED_HEXAGONAL_120   = 0b00001000;
		const CHECKED                 = 0b00010000;
    }
}

#[derive(PartialyEq, Eq, Clone)]
pub struct TileCell {
	tileset: Option<Rc<RefCell<Tileset>>>,
	id: isize,
	flags: TileIdFlags,
}

impl TileCell {
	pub fn new(tile: Option<&Tile>) Self {
		let flags = TileFlags::empty();
		let (tileset, id) = tile.map_or((None, -1),
				|t| (Some(tile.tileset.clone()), tile.id));
		Self {
			tileset, id,
			flags: TileFlags::empty(),
		}
	}

	pub fn is_empty(&self) -> bool { self.tileset.is_none() }
	pub fn tileset(&self) -> Option<Rc<RefCell<Tileset>>> { self.tileset() }
	pub fn id(&self) -> isize { self.id }

	pub fn tile(&self) -> Option<Rc<Tile>> {
		match self.tileset {
			Some(ts) => self.tileset.find_tile(self.id),
			None => None,
		}
	}

	//void setTile(Tile *tile);
	fn set_tile(tileset: Tileset, id: isize) {
		unimplemented!()
	}

	fn refers_tile(tile: &Tile) -> bool {
		self.tileset == tile.tileset() && self.id == tile.id()
	}
}


/*
inline Tile *Cell::tile() const
{
	return _tileset ? _tileset->findTile(_tileId) : nullptr;
}

inline void Cell::setTile(Tile *tile)
{
	if (tile)
		setTile(tile->tileset(), tile->id());
	else
		setTile(nullptr, -1);
}

inline void Cell::setTile(Tileset *tileset, int tileId)
{
	_tileset = tileset;
	_tileId = tileId;
}

inline bool Cell::refersTile(const Tile *tile) const
{
}
*/

/*
	bool flippedHorizontally() const { return _flippedHorizontally; }
	bool flippedVertically() const { return _flippedVertically; }
	bool flippedAntiDiagonally() const { return _flippedAntiDiagonally; }
	bool rotatedHexagonal120() const { return _rotatedHexagonal120; }

	void setFlippedHorizontally(bool f) { _flippedHorizontally = f; }
	void setFlippedVertically(bool f) { _flippedVertically = f; }
	void setFlippedAntiDiagonally(bool f) { _flippedAntiDiagonally = f; }
	void setRotatedHexagonal120(bool f) { _rotatedHexagonal120 = f; }

	bool checked() const { return _checked; }
	void setChecked(bool checked) { _checked = checked; }
*/

}