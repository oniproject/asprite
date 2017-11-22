use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use math::Vector2;
use specs::*;
use std::sync::Arc;
use vulkano::device::Queue;

use winit::Event;
use time::*;
use renderer::*;
use sprite::*;
use app::*;
use state;
use toml;

pub const RES: &str = "./ex/res.toml";

pub struct Velocity {
	pub vel: Vector2<f32>,
}

impl Component for Velocity {
	type Storage = VecStorage<Self>;
}

pub struct Scene {
	pub textures: Vec<Texture>,
	pub queue: Arc<Queue>,
	pub add: bool,
}

impl state::State<World, Event> for Scene {
	fn switch(&mut self, world: &mut World, event: state::ExecEvent) {
		use state::ExecEvent::*;
		println!("arena state change event: {:?}", event);
		match event {
			Start | Resume => {
				if self.textures.is_empty() {
					let (future, textures) = {
						use std::io::prelude::*;
						use std::fs::File;

						#[derive(Debug, Deserialize)]
						struct Assets {
							images: Vec<String>,
						}

						let mut f = File::open(RES).unwrap();
						let mut buffer = Vec::new();
						f.read_to_end(&mut buffer).unwrap();

						let decoded: Assets = toml::from_slice(&buffer).unwrap();
						println!("{:#?}", decoded);

						Texture::load_vec(self.queue.clone(), &decoded.images).unwrap()
					};

					self.textures = textures;

					world.write_resource::<Future>().join(future);
				};

				for _ in 0..300 {
					spawn(world, &self.textures);
				}
			}
			Stop | Pause => {
				world.delete_all();
			}
		}
	}

	fn update(&mut self, world: &mut World) -> SceneTransition<Event> {
		if self.add {
			for _ in 0..29 {
				spawn(world, &self.textures);
			}
		}
		None
	}

	fn fixed_update(&mut self, world: &mut World) -> SceneTransition<Event> {
		let gravity = 0.75;

		let dt = world.read_resource::<Time>().fixed_seconds;
		let size = *world.read_resource::<Vector2<f32>>();

		let mut speed = world.write::<Velocity>();
		let mut sprites = world.write::<Local>();

		let dt = dt * 50.0;
		let between = Range::new(0.0, 10.0);

		//use rayon::prelude::*;
		(&mut speed, &mut sprites).join().for_each(|(speed, sprite)| {
			let sprite = &mut sprite.0;
			let speed = &mut speed.vel;
			sprite.t += *speed * dt;
			speed.y += gravity * dt;

			if sprite.t.x > size.x {
				speed.x *= -1.0;
				sprite.t.x = size.x;
			} else if sprite.t.x < 0.0 {
				speed.x *= -1.0;
				sprite.t.x = 0.0;
			}

			if sprite.t.y > size.y {
				speed.y *= -0.85;
				sprite.t.y = size.y;
				let mut rng = thread_rng();
				if rng.gen() {
					speed.y -= between.ind_sample(&mut rng);
				}
			} else if sprite.t.y < 0.0 {
				speed.y = 0.0;
				sprite.t.y = 0.0;
			}
		});
		None
	}

	fn event(&mut self, _: &mut World, event: Event) -> SceneTransition<Event> {
		match event {
			Event::WindowEvent { event, .. } => {
				use winit::ElementState;
				use winit::WindowEvent::*;
				match event {
					Closed => {
						return Some(state::Transition::Pop);
					}
					MouseInput { state, .. } => {
						self.add = state == ElementState::Pressed;
					}
					_ => (),
				}
			}

			_ => (),
		}
		None
	}
}

fn spawn(world: &mut World, textures: &[Texture]) {
	let mut rng = thread_rng();
	let between = Range::new(0.0, 10.0);
	let tex = Range::new(0, textures.len());

	let x = between.ind_sample(&mut rng);
	let y = between.ind_sample(&mut rng) - 5.0;

	let tex = tex.ind_sample(&mut rng);
	let t = &textures[tex];

	let mut sprite = Sprite::new(t.clone());
	sprite.anchor.y = 1.0;
	sprite.size.x = t.wh.0 as f32;
	sprite.size.y = t.wh.1 as f32;

	let speed = Velocity {
		vel: Vector2::new(x, y),
	};

	let local = Local::default();
	let global = Global::default();

	world.create_entity()
		.with(sprite)
		.with(local)
		.with(global)
		.with(speed)
		.build();
}
