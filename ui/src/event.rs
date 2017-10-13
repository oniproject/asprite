use super::*;

#[derive(Copy, Clone, Debug)]
pub enum Event<N: Num> {
	Init,
	Mouse {
		point: Point<N>,
		left: bool,
		middle: bool,
		right: bool,
	},
	Scroll {
		x: N,
		y: N,
	},
	Text {
		c: char,
	},
	Enter,
	Backspace,
	Delete,
	Home,
	End,
	UpArrow,
	DownArrow,
	LeftArrow,
	RightArrow,
	Resize {
		width: u32,
		height: u32,
	},
	Unknown,
}