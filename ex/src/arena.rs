use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use math::Vector2;
use specs::{Component, VecStorage};
use specs::{LazyUpdate, Entities, System, WriteStorage, Join, Fetch, FetchMut};

use std::time::Duration;

use renderer::*;
use sprite::*;

pub struct Velocity {
	pub vel: Vector2<f32>,
}

impl Component for Velocity {
	type Storage = VecStorage<Self>;
}

pub struct Arena {
	pub textures: Vec<Texture>,
	pub gravity: f32,
	pub wh: (f32, f32),
}

impl Arena {
	pub fn new(textures: Vec<Texture>) -> Self {
		Self {
			textures,
			gravity: 0.75,
			wh: (25.0, 32.0),
		}
	}

	pub fn spawn_lazy<'a>(&self, e: &Entities<'a>, lazy: &Fetch<'a, LazyUpdate>) {
		let mut rng = thread_rng();
		let between = Range::new(0.0, 10.0);
		let tex = Range::new(0, self.textures.len());

		let x = between.ind_sample(&mut rng);
		let y = between.ind_sample(&mut rng) - 5.0;

		let tex = tex.ind_sample(&mut rng);
		let t = &self.textures[tex];

		let mut sprite = Sprite::new(self.textures[tex].clone());
		sprite.anchor.y = 1.0;
		sprite.size.x = t.wh.0 as f32;
		sprite.size.y = t.wh.1 as f32;

		let speed = Velocity {
			vel: Vector2::new(x, y),
		};

		let transform = SpriteTransform::default();

		let e = e.create();
		lazy.insert(e, sprite);
		lazy.insert(e, transform);
		lazy.insert(e, speed);
	}
}

/*
pub struct Time {
	pub d: Duration,
}

impl Time {
	#[inline(always)]
	fn to_secs(&self) -> f32 {
		self.d.as_secs() as f32 + self.d.subsec_nanos() as f32 / 1.0e9
	}
	#[inline(always)]
	fn to_nanos(&self) -> u64 {
		(self.d.as_secs() * 1_000_000_000) + self.d.subsec_nanos() as u64
	}
}
*/

#[inline(always)]
pub fn duration_to_secs(d: Duration) -> f32 {
	d.as_secs() as f32 + (d.subsec_nanos() as f32 / 1.0e9)
}
/*
#[inline(always)]
fn duration_to_nanos(d: Duration) -> u64 {
	(d.as_secs() * 1_000_000_000) + d.subsec_nanos() as u64
}
*/

impl<'a> System<'a> for Arena {
	type SystemData = (
		Entities<'a>,
		Fetch<'a, LazyUpdate>,
		FetchMut<'a, usize>,
		Fetch<'a, Vector2<f32>>,
		Fetch<'a, Duration>,
		WriteStorage<'a, Velocity>,
		WriteStorage<'a, SpriteTransform>,
	);
	fn run(&mut self, (e, lazy, mut add, size, dt, mut speed, mut sprites): Self::SystemData) {
		//use rayon::prelude::*;

		if *add != 0 {
			for _ in 0..*add {
				self.spawn_lazy(&e, &lazy);
			}
			*add = 0;
		}

		let dt = duration_to_secs(*dt) * 50.0;
		let size = *size;
		let between = Range::new(0.0, 10.0);

		(&mut speed, &mut sprites).join().for_each(|(speed, sprite)| {
			let sprite = &mut sprite.0;
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
				let mut rng = thread_rng();
				if rng.gen() {
					speed.y -= between.ind_sample(&mut rng);
				}
			} else if sprite.t.y < 0.0 {
				speed.y = 0.0;
				sprite.t.y = 0.0;
			}
		});
	}
}
