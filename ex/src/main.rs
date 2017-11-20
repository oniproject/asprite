#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(derive_clone_copy)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]

extern crate renderer;
extern crate math;

extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate specs;
extern crate hibitset;
extern crate fnv;

extern crate rayon;
extern crate rand;

#[macro_use] extern crate derivative;

extern crate winit;
extern crate vulkano;
extern crate vulkano_win;

use vulkano_win::VkSurfaceBuild;
use vulkano::sync::GpuFuture;
use vulkano::instance::{Instance, PhysicalDevice};

use specs::World;

use std::time::{Instant, Duration};

use math::*;

mod arena;
mod input;
mod sprite;
mod tsys;
mod state;
mod sprite_batcher;

use sprite::*;
use renderer::*;
use sprite_batcher::*;

pub const TEXTURE_COUNT: u32 = 16;
pub const BATCH_CAPACITY: usize = 2_000;

pub const RES: &str = "./ex/res.toml";

fn main() {

	let extensions = vulkano_win::required_extensions();
	let instance = Instance::new(None, &extensions, &[])
		.expect("failed to create instance");

	let physical = PhysicalDevice::enumerate(&instance)
		.next().expect("no device available");
	println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

	let events_loop = winit::EventsLoop::new();
	let window = winit::WindowBuilder::new()
		.build_vk_surface(&events_loop, instance.clone())
		.expect("can't build window");

	let (chain, images) = Chain::new(physical, window, |caps| caps.supported_formats[0].0);

	let (textures_future, textures) = {
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

		Texture::load_vec(chain.queue.clone(), &decoded.images).unwrap()
	};

	let (mut world, mut dispatcher) = {
		println!("create world");

		let mut world = World::new();
		world.register::<Sprite>();
		world.register::<SpriteTransform>();
		world.register::<arena::Velocity>();

		let arena = arena::Arena::new(textures.clone());
		let input = input::InputSystem { events_loop, add: false };

		let (buf, index_future) = Batcher::new(BATCH_CAPACITY, TEXTURE_COUNT, chain, &images);

		let future: Box<GpuFuture + Send + Sync> = Box::new(textures_future.join(index_future));

		world.add_resource(Future::new(future));
		world.add_resource(BATCH_CAPACITY as usize);
		world.add_resource(Vector2::new(1024.0f32, 786.0));
		world.add_resource(Duration::default());

		let dispatcher = specs::DispatcherBuilder::new()
			.add(arena, "mark", &[])
			.add(input, "input", &[])
			.add(SpriteSystem, "sprite", &["mark"])
			.add(buf, "batcher", &["sprite"])
			.build();

		(world, dispatcher)
	};

	println!();
	println!("start");

	let mut ticker = Ticker::new();
	loop {
		ticker.update();
		world.add_resource(ticker.elapsed);

		dispatcher.dispatch(&mut world.res);
		world.maintain();
	}
}

struct Ticker {
	now: Instant,
	times: Vec<Duration>,
	elapsed: Duration,
}

impl Ticker {
	fn new() -> Self {
		Self {
			elapsed: Duration::new(0, 0),
			now: Instant::now(),
			times: Vec::new(),
		}
	}

	fn update(&mut self) {
		self.elapsed = self.now.elapsed();
		self.times.push(self.elapsed);
		self.now = Instant::now();

		if self.times.len() >= 60 {
			println!("{}", {
				let sum: Duration = self.times.drain(..).sum();
				let sum = sum;
				let s = (sum.as_secs() * 1000) as f64;
				let n = sum.subsec_nanos() as f64 / 1_000_000.0;
				(s + n) / 60.0
			});
		}
	}
}

/*
#[macro_use]
extern crate vulkano_shader_derive;
mod vs {
	#[derive(VulkanoShader)]
	#[ty = "vertex"]
	#[src = "

#version 450

precision highp float;

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 tex_coords;
layout(location = 1) out vec4 tex_color;

layout(set = 0, binding = 0) uniform uni {
	mat4 proj;
	vec4 color;
} uniforms;

void main() {
	mat3 proj = mat3(uniforms.proj);
	vec2 pos = (proj * vec3(position, 1.0)).xy;
	gl_Position = vec4(pos, 0.0, 1.0);
	tex_coords = uv;
	tex_color = uniforms.color;
}

"]
	struct Dummy;
}

mod fs {
	#[derive(VulkanoShader)]
	#[ty = "fragment"]
	#[src = "
#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 1) in vec4 tex_color;
layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler2D tex;

void main() {
	f_color = vec4(1.0, 1.0, 1.0, texture(tex, tex_coords).r) * tex_color;
}
"]
	struct Dummy;
}
*/
