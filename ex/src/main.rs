#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(derive_clone_copy)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]

#[cfg(feature = "profiler")]
#[macro_use] extern crate thread_profiler;

extern crate renderer;
extern crate math;
extern crate ui;

extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate specs;
extern crate hibitset;
extern crate fnv;

extern crate rayon;
extern crate rand;

#[macro_use] extern crate derivative;
#[macro_use] extern crate lazy_static;

extern crate winit;
extern crate vulkano;
extern crate vulkano_win;

mod app;
mod arena;
mod sprite;
mod tsys;
mod state;
mod sprite_batcher;

use math::*;
use app::*;
use sprite_batcher::*;

pub const TEXTURE_COUNT: u32 = 16;
pub const BATCH_CAPACITY: usize = 2_000;

/*
fn draw() {
	fn draw_indexed(
		//&mut self,
		index_count: u32,
		instance_count: u32,
		first_index: u32,
		vertex_offset: i32,
		first_instance: u32
	) {}

	let from = N;
	let count = N;

	let index_offset = from * INDEX_BY_QUAD;
	let vertex_offset = from * VERTEX_BY_QUAD;

	draw_indexed(count, 1, index_offset, vertex_offset, 0);
}
*/

fn main() {
	let (mut events_loop, b) = BatcherBundle::new();


	let queue = b.batcher.queue.clone();

	let mut app = {
		println!("create app");

		use std::sync::Arc;
		let conf = rayon::Configuration::new()
			.start_handler(|n| {
				#[cfg(feature = "profiler")]
				{
					let name = format!("th#{}", n);
					::thread_profiler::register_thread_with_profiler(name.into());
				}
			});
		let pool = Arc::new(rayon::ThreadPool::new(conf).unwrap());

		let dispatcher = specs::DispatcherBuilder::new()
			.with_pool(pool.clone())
			.add(sprite::TransformSystem::default(), "transform", &[]);

		let mut world = specs::World::new();
		world.register::<arena::Velocity>();
		world.add_resource(pool);

		let dispatcher = b.bundle(&mut world, dispatcher);

		App::new(world, dispatcher.build())
	};

	app.world.write_resource::<Time>().set_fixed_time(::std::time::Duration::new(0, 16666666*4));

	println!();
	println!("run");

	let arena = arena::Scene { textures: Vec::new(), queue, add: false };
	app.run(Box::new(arena), |world, states|
		events_loop.poll_events(|event| states.event(world, event))
	);
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
