use std::ops::Range;

/// Convenience function for creating tile terrain information.
#[inline(always)]
pub fn make_terrain(top_left: u32, top_right: u32, bottom_left: u32, bottom_right: u32) -> u32 {
	(top_left & 0xFF) << 24 |
	(top_right & 0xFF) << 16 |
	(bottom_left & 0xFF) << 8 |
	(bottom_right & 0xFF)
}

/// Returns the given terrain with the corner modified to terrain_id.
#[inline(always)]
pub fn set_terrain_corner(terrain: u32, corner: u32, terrain_id: u32) -> u32 {
	let mask = 0xFF << (3 - corner) * 8;
	let insert = terrain_id << (3 - corner) * 8;
	(terrain & !mask) | (insert & mask)
}

/// A single frame of an animated tile.
#[derive(PartialEq, Eq)]
pub struct Frame {
	pub tile_id: isize,
	pub duration: isize,
}

pub type TileId = isize;

pub struct Tilesets {
	pub tilesets: Vec<Tileset>,
}

impl Tilesets {
	pub fn contains_id(&self, id: TileId) -> bool {
		self.tilesets.iter().any(|ts| ts.range.contains(id))
	}
	pub fn find_tileset(&self, id: TileId) -> Option<&Tileset> {
		self.tilesets.iter().find(|ts| ts.range.contains(id))
	}
}

pub struct Tileset {
	pub range: Range<isize>,
	pub name: String,
	pub tile_width: usize,
	pub tile_height: usize,
	/*
	QString mName;
	QString mFileName;
	ImageReference mImageReference;
	int mTileWidth;
	int mTileHeight;
	int mTileSpacing;
	int mMargin;
	QPoint mTileOffset;
	Orientation mOrientation;
	QSize mGridSize;
	int mColumnCount;
	int mExpectedColumnCount;
	int mExpectedRowCount;
	QMap<int, Tile*> mTiles;
	int mNextTileId;
	QList<Terrain*> mTerrainTypes;
	QList<WangSet*> mWangSets;
	int mMaximumTerrainDistance;
	bool mTerrainDistancesDirty;
	LoadingStatus mStatus;
	QColor mBackgroundColor;
	QPointer<TilesetFormat> mFormat;
	*/
}

impl Tileset {
	pub fn new(name: String, start_id: TileId, count: TileId) -> Self {
		Self {
			name,
			range: Range { start: start_id, end: start_id + count },
			tile_width: 0,
			tile_height: 0,
		}
	}
	pub fn contains_id(&self, id: TileId) -> bool {
		self.range.contains(id)
	}
}

#[test]
fn contains_id() {
	let a = Tileset::new("first", 1, 5);
	let b = Tileset::new("second", 8, 2);
	let ts = Tilesets {
		tilesets: vec![a, b],
	};

	assert!(!ts.contains_id(0));
	assert!(ts.contains_id(1));
	assert!(ts.contains_id(2));
	assert!(ts.contains_id(3));
	assert!(ts.contains_id(4));
	assert!(ts.contains_id(5));
	assert!(!ts.contains_id(6));
	assert!(!ts.contains_id(7));
	assert!(ts.contains_id(8));
	assert!(ts.contains_id(9));
	assert!(!ts.contains_id(10));
}