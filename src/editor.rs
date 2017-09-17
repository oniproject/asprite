use common::*;
use tool::*;

pub struct SimpleContext {
	pub grid: Vec<u8>, 
	pub size: Point<i16>,
	pub pos: Point<i16>,

	pub mx: i32,
	pub my: i32,

	pub mouse: Point<i16>,
	pub zoom: i16,
	pub fg: u8,
	pub bg: u8,
}

impl SimpleContext {
	pub fn new(zoom: i16, pos: Point<i16>, size: Point<i16>) -> Self {
		Self {
			zoom, pos, size,
			grid: vec![0; (size.x * size.y) as usize],
			bg: 0,
			fg: 1,
			mouse: Point::new(0, 0),
			mx: 0, my: 0,
		}
	}

	pub fn set_mouse(&mut self, x: i32, y: i32) -> Point<i16> {
		self.mx = x;
		self.my = y;
		let x = (x as i16 - self.pos.x) / self.zoom;
		let y = (y as i16 - self.pos.y) / self.zoom;
		self.mouse = Point::new(x, y);
		self.mouse
	}
}

impl Context for SimpleContext {
	fn start(&mut self) -> u8 { self.fg }
	fn commit(&mut self) {}
	fn rollback(&mut self) {}

	fn brush(&mut self, p: Point<i16>, color: u8) {
		let x = p.x >= 0 && p.x < self.size.x;
		let y = p.y >= 0 && p.y < self.size.y;
		if x && y {
			let idx = p.x + p.y * self.size.x;
			self.grid[idx as usize] = color;
		}
	}
}