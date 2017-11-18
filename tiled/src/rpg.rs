#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TileId {
	pub id: i16,
}

pub enum Source { A1,  A2,  A3,  A4,  A5,  B, C, D, E }

const B: i16   = 0;
const C: i16   = 256;
const D: i16   = 512;
const E: i16   = 768;
const A5: i16  = 1536;
const A1: i16  = 2048;
const A2: i16  = 2816;
const A3: i16  = 4352;
const A4: i16  = 5888;
const MAX: i16 = 8192;

impl TileId {
	pub fn is_visible(self) -> bool {
		self.id > 0 && self.id < MAX
	}
	pub fn is_autotile(self) -> bool {
		self.id >= A1
	}
	pub fn autotile_kind(self) -> i16 {
		(self.id - A1) / 48
	}
	pub fn autotile_shape(self) -> i16 {
		(self.id - A1) % 48
	}
	pub fn make_autotile(kind: i16, shape: i16) -> Self {
		Self { id: A1 + kind * 48 + shape }
	}
	pub fn is_same_kind(self, other: Self) -> bool {
		if self.is_autotile() && other.is_autotile() {
			self.autotile_kind() == other.autotile_kind()
		} else {
			self.id == other.id
		}
	}

	pub fn source(self) -> Option<Source> {
		match self.id {
			id if { id >= A1 && id < A2  } => Some(Source::A1),
			id if { id >= A2 && id < A3  } => Some(Source::A2),
			id if { id >= A3 && id < A4  } => Some(Source::A3),
			id if { id >= A4 && id < MAX } => Some(Source::A4),
			id if { id >= A5 && id < A1  } => Some(Source::A5),
			id if { id >= B  && id < C   } => Some(Source::B),
			id if { id >= C  && id < D   } => Some(Source::C),
			id if { id >= D  && id < E   } => Some(Source::D),
			id if { id >= E  && id < A5  } => Some(Source::E),
			_ => None,
		}
	}

	pub fn is_a1(self) -> bool { self.id >= A1 && self.id < A2 }
	pub fn is_a2(self) -> bool { self.id >= A2 && self.id < A3 }
	pub fn is_a3(self) -> bool { self.id >= A3 && self.id < A4 }
	pub fn is_a4(self) -> bool { self.id >= A4 && self.id < MAX }
	pub fn is_a5(self) -> bool { self.id >= A5 && self.id < A1 }
	pub fn is_b(self) -> bool  { self.id >= B  && self.id < C }
	pub fn is_c(self) -> bool  { self.id >= C  && self.id < D }
	pub fn is_d(self) -> bool  { self.id >= D  && self.id < E }
	pub fn is_e(self) -> bool  { self.id >= E  && self.id < A5 }

	pub fn is_water(self) -> bool {
		self.is_a1() && !(self.id >= A1 + 96 && self.id < A1 + 192)
	}
	pub fn is_waterfall(self) -> bool {
		(self.id >= A1 + 192 && self.id < A2) && self.autotile_kind() % 2 == 1
	}
	pub fn is_ground(self) -> bool {
		self.is_a1() || self.is_a2() || self.is_a5()
	}
	pub fn is_shadowing(self) -> bool {
		self.is_a3() || self.is_a4()
	}
	pub fn is_roof(self) -> bool {
		self.is_a3() && self.autotile_kind() % 16 < 8
	}
	pub fn is_wall_top(self) -> bool {
		self.is_a4() && self.autotile_kind() % 16 < 8
	}
	pub fn is_wall_side(self) -> bool {
		(self.is_a3() || self.is_a4()) && self.autotile_kind() % 16 >= 8
	}
	pub fn is_wall(self) -> bool {
		self.is_wall_top() || self.is_wall_side()
	}
	pub fn is_floor_type_autotile(self) -> bool {
		(self.is_a1() && !self.is_waterfall()) || self.is_a2() || self.is_wall_top()
	}
	pub fn is_wall_type_autotile(self) -> bool {
		self.is_roof() || self.is_wall_side()
	}
	pub fn is_waterfall_type_autotile(self) -> bool {
		self.is_waterfall()
	}
}

/*
// Autotile shape number to coordinates of tileset images

Tilemap.FLOOR_AUTOTILE_TABLE = [
	[[2,4],[1,4],[2,3],[1,3]],[[2,0],[1,4],[2,3],[1,3]],
	[[2,4],[3,0],[2,3],[1,3]],[[2,0],[3,0],[2,3],[1,3]],
	[[2,4],[1,4],[2,3],[3,1]],[[2,0],[1,4],[2,3],[3,1]],
	[[2,4],[3,0],[2,3],[3,1]],[[2,0],[3,0],[2,3],[3,1]],
	[[2,4],[1,4],[2,1],[1,3]],[[2,0],[1,4],[2,1],[1,3]],
	[[2,4],[3,0],[2,1],[1,3]],[[2,0],[3,0],[2,1],[1,3]],
	[[2,4],[1,4],[2,1],[3,1]],[[2,0],[1,4],[2,1],[3,1]],
	[[2,4],[3,0],[2,1],[3,1]],[[2,0],[3,0],[2,1],[3,1]],
	[[0,4],[1,4],[0,3],[1,3]],[[0,4],[3,0],[0,3],[1,3]],
	[[0,4],[1,4],[0,3],[3,1]],[[0,4],[3,0],[0,3],[3,1]],
	[[2,2],[1,2],[2,3],[1,3]],[[2,2],[1,2],[2,3],[3,1]],
	[[2,2],[1,2],[2,1],[1,3]],[[2,2],[1,2],[2,1],[3,1]],
	[[2,4],[3,4],[2,3],[3,3]],[[2,4],[3,4],[2,1],[3,3]],
	[[2,0],[3,4],[2,3],[3,3]],[[2,0],[3,4],[2,1],[3,3]],
	[[2,4],[1,4],[2,5],[1,5]],[[2,0],[1,4],[2,5],[1,5]],
	[[2,4],[3,0],[2,5],[1,5]],[[2,0],[3,0],[2,5],[1,5]],
	[[0,4],[3,4],[0,3],[3,3]],[[2,2],[1,2],[2,5],[1,5]],
	[[0,2],[1,2],[0,3],[1,3]],[[0,2],[1,2],[0,3],[3,1]],
	[[2,2],[3,2],[2,3],[3,3]],[[2,2],[3,2],[2,1],[3,3]],
	[[2,4],[3,4],[2,5],[3,5]],[[2,0],[3,4],[2,5],[3,5]],
	[[0,4],[1,4],[0,5],[1,5]],[[0,4],[3,0],[0,5],[1,5]],
	[[0,2],[3,2],[0,3],[3,3]],[[0,2],[1,2],[0,5],[1,5]],
	[[0,4],[3,4],[0,5],[3,5]],[[2,2],[3,2],[2,5],[3,5]],
	[[0,2],[3,2],[0,5],[3,5]],[[0,0],[1,0],[0,1],[1,1]]
];

Tilemap.WALL_AUTOTILE_TABLE = [
	[[2,2],[1,2],[2,1],[1,1]],[[0,2],[1,2],[0,1],[1,1]],
	[[2,0],[1,0],[2,1],[1,1]],[[0,0],[1,0],[0,1],[1,1]],
	[[2,2],[3,2],[2,1],[3,1]],[[0,2],[3,2],[0,1],[3,1]],
	[[2,0],[3,0],[2,1],[3,1]],[[0,0],[3,0],[0,1],[3,1]],
	[[2,2],[1,2],[2,3],[1,3]],[[0,2],[1,2],[0,3],[1,3]],
	[[2,0],[1,0],[2,3],[1,3]],[[0,0],[1,0],[0,3],[1,3]],
	[[2,2],[3,2],[2,3],[3,3]],[[0,2],[3,2],[0,3],[3,3]],
	[[2,0],[3,0],[2,3],[3,3]],[[0,0],[3,0],[0,3],[3,3]]
];

Tilemap.WATERFALL_AUTOTILE_TABLE = [
	[[2,0],[1,0],[2,1],[1,1]],[[0,0],[1,0],[0,1],[1,1]],
	[[2,0],[3,0],[2,1],[3,1]],[[0,0],[3,0],[0,1],[3,1]]
];
*/