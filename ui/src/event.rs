use super::*;

#[derive(Copy, Clone, Debug)]
pub enum Event<N: BaseNum> {
	Init,
	Mouse {
		point: Point2<N>,
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