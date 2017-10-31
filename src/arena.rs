use rand;
use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use cgmath::Vector2;
use specs::{Component, VecStorage, World};
use specs::{System, WriteStorage, ParJoin, Fetch};

use rayon::iter::ParallelIterator;

use std::time::Duration;

use texture::*;
use sprite::*;

pub struct Velocity {
	pub vel: Vector2<f32>,
}

impl Component for Velocity {
	type Storage = VecStorage<Self>;
}

pub struct Arena {
	pub textures: Vec<BaseTexture>,
	pub gravity: f32,
	pub wh: (f32, f32),
}

impl Arena {
	pub fn new(textures: Vec<BaseTexture>) -> Self {
		Self {
			textures,
			gravity: 0.75,
			wh: (25.0, 32.0),
		}
	}

	pub fn spawn(&mut self, w: &mut World) {
		let mut rng = rand::thread_rng();
		let between = Range::new(0.0, 10.0);
		let tex = Range::new(0, self.textures.len());

		let x = between.ind_sample(&mut rng);
		let y = between.ind_sample(&mut rng) - 5.0;

		let tex = tex.ind_sample(&mut rng);
		let t = &self.textures[tex];

		let mut sprite = Sprite {
			anchor: Vector2::new(0.5, 1.0),
			w: t.wh.0 as f32, h: t.wh.1 as f32,
			.. Default::default()
		};
		sprite.uv();
		sprite.set_texture(tex as u32);

		sprite.recalc();

		let speed = Velocity {
			vel: Vector2::new(x, y),
		};

		w.create_entity()
			.with(sprite)
			.with(speed)
			.build();
	}
}

#[inline(always)]
fn duration_to_secs(d: Duration) -> f32 {
	d.as_secs() as f32 + (d.subsec_nanos() as f32 / 1.0e9)
}
#[inline(always)]
fn duration_to_nanos(d: Duration) -> u64 {
	(d.as_secs() * 1_000_000_000) + d.subsec_nanos() as u64
}

impl<'a> System<'a> for Arena {
	type SystemData = (
		Fetch<'a, Vector2<f32>>,
		Fetch<'a, Duration>,
		WriteStorage<'a, Velocity>,
		WriteStorage<'a, Sprite>,
	);
	fn run(&mut self, (size, dt, mut speed, mut sprites): Self::SystemData) {
		let dt = duration_to_secs(*dt) * 50.0;
		let size = *size;
		let between = Range::new(0.0, 10.0);
		(&mut speed, &mut sprites).par_join().for_each(|(speed, sprite)| {
			let mut rng = thread_rng();
			let speed = &mut speed.vel;
			sprite.t += *speed * dt;
			speed.y += self.gravity * dt;

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
				if rng.gen() {
					speed.y -= between.ind_sample(&mut rng);
				}
			} else if sprite.t.y < 0.0 {
				speed.y = 0.0;
				sprite.t.y = 0.0;
			}
			sprite.recalc();
		});
	}
}