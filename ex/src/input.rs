
enum Event {
	Key(winit::KeyboardInput),
	Scroll(Vector2<f32>),
	Mouse {
		point: Point2<f32>,
		left: bool,
		middle: bool,
		right: bool,
	},
	Char(char),
	Closed,
}

struct InputManager {
	point: Point2<f32>,
	left: bool,
	middle: bool,
	right: bool,
}

impl InputManager {
	fn mouse(&self) -> Event {
		Event::Mouse {
			point: self.point,
			left: self.left,
			middle: self.middle,
			right: self.right,
		}
	}

	fn ev(&mut self, ev: winit::Event) -> Option<Event> {
		use winit::Event::*;
		match ev {
			WindowEvent { event, .. } => self.win(event).ok(),
			DeviceEvent { event, .. } => {
				match event {
					winit::DeviceEvent::Text { codepoint } =>
						Some(Event::Char(codepoint)),
					_ => None,
				}
			}
			_ => None,
		}
	}

	fn win(&mut self, ev: winit::WindowEvent) -> Result<Event, winit::WindowEvent> {
		use winit::WindowEvent::*;
		use winit::ElementState;
		use winit::MouseButton;
		match ev {
			Closed => { Ok(Event::Closed) }
			MouseMoved { position, .. } => {
				self.point.x = position.0 as f32;
				self.point.y = position.0 as f32;
				Ok(self.mouse())
			}
			MouseInput { state, button, .. } => {
				let is = state == ElementState::Pressed;
				match button {
					MouseButton::Left =>   { self.left = is;   Ok(self.mouse()) }
					MouseButton::Middle => { self.middle = is; Ok(self.mouse()) }
					MouseButton::Right =>  { self.right = is;  Ok(self.mouse()) }
					_ => Err(ev),
				}
			}
			MouseWheel { delta: winit::MouseScrollDelta::LineDelta(x, y), .. } => {
				Ok(Event::Scroll(Vector2::new(x, y).into()))
			}
			KeyboardInput { input, .. } => {
				Ok(Event::Key(input))
			}
			_ => Err(ev),
		}
	}
}
