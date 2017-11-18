#![allow(dead_code)]

use sdl2;
pub use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;

use either::{Either, Left, Right};
use sdl2::event::Event as Ev;
use sdl2::mouse::MouseButton;

pub enum Event {
	Move(i16, i16, i16, i16),
	Btn(i16, i16, Btn, Dir),
	Wheel(i16, i16),
	Press(Key),
	Release(Key),
}

pub enum Btn {
	Left,
	Right,
	Middle,
}

pub enum Dir {
	Press, Release,
}

pub enum Key {
    Cancel,
	ZoomIn,
	ZoomOut,

	NextWidget,
	PrevWidget,

	Shift,
	Ctrl,
	Alt,

	Other(Keycode),
}

fn convert_key(key: Keycode, keymod: Mod) -> Key {
	let shift = keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD);
	let _alt = keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD);
	let _ctrl = keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD);
	match key {
	Keycode::Plus  | Keycode::KpPlus  => Key::ZoomIn,
	Keycode::Minus | Keycode::KpMinus => Key::ZoomOut,

	Keycode::Tab if shift => Key::PrevWidget,
	Keycode::Tab if !shift => Key::NextWidget,

	Keycode::Escape => Key::Cancel,

	Keycode::LShift |
	Keycode::RShift => Key::Shift,

	Keycode::LCtrl |
	Keycode::RCtrl => Key::Ctrl,

	Keycode::LAlt |
	Keycode::RAlt => Key::Alt,

	_ => Key::Other(key),
	}
}

pub fn convert(e: Ev) -> Either<Event, Ev> {
	match e {
		Ev::MouseMotion { x, y, xrel, yrel, .. } => Left(Event::Move(x as i16, y as i16, xrel as i16, yrel as i16)),
		Ev::MouseWheel { x, y, .. } => Left(Event::Wheel(x as i16, y as i16)),

		Ev::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => Left(Event::Btn(x as i16, y as i16, Btn::Left, Dir::Press)),
		Ev::MouseButtonDown { x, y, mouse_btn: MouseButton::Middle, .. } => Left(Event::Btn(x as i16, y as i16, Btn::Middle, Dir::Press)),
		Ev::MouseButtonDown { x, y, mouse_btn: MouseButton::Right, .. } => Left(Event::Btn(x as i16, y as i16, Btn::Right, Dir::Press)),

		Ev::MouseButtonUp { x, y, mouse_btn: MouseButton::Left, .. } => Left(Event::Btn(x as i16, y as i16, Btn::Left, Dir::Release)),
		Ev::MouseButtonUp { x, y, mouse_btn: MouseButton::Middle, .. } => Left(Event::Btn(x as i16, y as i16, Btn::Middle, Dir::Release)),
		Ev::MouseButtonUp { x, y, mouse_btn: MouseButton::Right, .. } => Left(Event::Btn(x as i16, y as i16, Btn::Right, Dir::Release)),

		Ev::KeyDown { keycode: Some(key), keymod, .. } => Left(Event::Press(convert_key(key, keymod))),
		Ev::KeyUp { keycode: Some(key), keymod, .. } => Left(Event::Release(convert_key(key, keymod))),
		_ => Right(e),
	}
}