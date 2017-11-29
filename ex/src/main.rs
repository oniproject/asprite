#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(derive_clone_copy)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]
#![feature(const_cell_new)]
#![feature(generators, generator_trait)]

#[cfg(feature = "profiler")]
#[macro_use] extern crate thread_profiler;

#[macro_use] extern crate error_chain;

extern crate lyon;

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

extern crate vulkano;

mod graphics;

mod loader;

mod app;
mod arena;
mod sprite;
mod tsys;
mod state;
mod sprite_batcher;

use math::*;
use app::*;
use sprite_batcher::*;
use renderer::*;

pub const TEXTURE_COUNT: u32 = 16;
pub const BATCH_CAPACITY: usize = 2_000;

#[inline]
pub fn mouse_event_buttons(mouse: &mut ui::Mouse, state: winit::ElementState, button: winit::MouseButton) {
	use winit::ElementState::*;
	use winit::MouseButton::*;
	let id = match button {
		Left => 0,
		Middle => 1,
		Right => 2,
		_ => return,
	};
	mouse.pressed[id] = state == Pressed;
	mouse.released[id] = state == Released;
}

#[inline]
pub fn mouse_event_movement(mouse: &mut ui::Mouse, position: (f64, f64)) {
	let x = position.0 as f32;
	let y = position.1 as f32;
	mouse.cursor = Point2::new(x, y);
}

fn main() {
	let (mut events_loop, b) = BatcherBundle::new();

	let queue = b.batcher.queue.clone();

	println!("create app");

	use std::sync::Arc;
	let conf = rayon::Configuration::new()
		.start_handler(|_num| {
			#[cfg(feature = "profiler")]
			{
				let name = format!("th#{}", _num);
				::thread_profiler::register_thread_with_profiler(name.into());
			}
		});
	let pool = Arc::new(rayon::ThreadPool::new(conf).unwrap());

	let dispatcher = specs::DispatcherBuilder::new()
		.with_pool(pool.clone());

	let mut world = specs::World::new();
	world.register::<arena::Velocity>();
	world.add_resource(pool);
	world.add_resource(ui::Mouse::new());

	let dispatcher = b.bundle(&mut world, dispatcher);

	let mut app = App::new(world, dispatcher.build());
	app.world.write_resource::<Time>().set_fixed_time(::std::time::Duration::new(0, 16666666*2));

	println!();
	println!("run");

	let arena = arena::Scene { textures: Vec::new(), queue };
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
