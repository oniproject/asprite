#![allow(unused_variables)]

use winit::EventsLoop;
use specs::Entities;
use specs::System;
use specs::FetchMut;

pub struct InputSystem {
	pub events_loop: EventsLoop,
	pub add: bool,
}

impl<'a> System<'a> for InputSystem {
	type SystemData = (Entities<'a>, FetchMut<'a, usize>);

	fn run(&mut self, (e, mut add_count): Self::SystemData) {
		let add = &mut self.add;
		self.events_loop.poll_events(|ev| {
			use winit::{Event, WindowEvent, ElementState};
			//use winit::VirtualKeyCode as VK;
			match ev {
				Event::WindowEvent { event, window_id } => {
					// println!("win#{:?} {:?}", window_id, event);
					match event {
						WindowEvent::Closed => {
							use specs::Join;
							println!("count: {}", e.join().count());
							::std::process::exit(0)
						}
						WindowEvent::MouseInput { state, .. } => {
							*add = state == ElementState::Pressed;
						}
						_ => (),
					}
				}
				Event::DeviceEvent { event, device_id } => {
					// println!("dev#{:?} {:?}", device_id, event);
				}
				Event::Awakened => println!("Awakened"),
				// TODO: Event::Suspended(is) => println!("Suspended {}", is),

				/*
				Event::WindowEvent { event: WindowEvent::KeyboardInput {
					input: winit::KeyboardInput {
						state: winit::ElementState::Pressed,
						virtual_keycode: Some(key),
						//modifiers: winit::ModifiersState {shift, ..},
						..
					},
					..
				}, .. } => {
					match key {
						VK::W => sprite.t.y -= 10.0,
						VK::S => sprite.t.y += 10.0,
						VK::A => sprite.t.x -= 10.0,
						VK::D => sprite.t.x += 10.0,
						_ => (),
					}
				},
				*/
			}
		});

		*add_count = if *add { 100 } else { 0 };
	}
}

