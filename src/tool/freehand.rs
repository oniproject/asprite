use common::*;

pub enum Input {
	Press(Point<i16>),
	Release(Point<i16>),
	Move(Point<i16>),
	Cancel, // press ESC
}

pub enum Mode {
	Continious,
	PixelPerfect,
	Disontinious,
	Single,
}

pub struct Op {
	pub mode: Mode,
	pub last: Option<Point<i16>>,
}

impl Op {
	pub fn run(&mut self, input: Input) {
		match input {
			Input::Press(_p) => (),
			Input::Release(_p) => (),
			Input::Move(_p) => (),
			Input::Cancel => (),
		}
	}
}