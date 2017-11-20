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

use math::*;

mod arena;
mod sprite;
mod tsys;
mod state;
mod sprite_batcher;

mod time;

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

	{
		println!("Using device: {} (type: {:?})", physical.name(), physical.ty());
		println!();
		// Enumerating memory heaps.
		for heap in physical.memory_heaps() {
			println!("Heap #{:?} has a capacity of {:?} bytes", heap.id(), heap.size());
		}
		println!();
		// Enumerating memory types.
		for ty in physical.memory_types() {
			println!("Memory type belongs to heap #{:?}", ty.heap().id());
			println!("Host-accessible: {:?}", ty.is_host_visible());
			println!("Device-local: {:?}", ty.is_device_local());
		}
		println!();
	}

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

	let mut app = {
		println!("create app");

		let mut world = World::new();
		world.register::<Sprite>();
		world.register::<SpriteTransform>();
		world.register::<arena::Velocity>();

		let (buf, index_future) = Batcher::new(BATCH_CAPACITY, TEXTURE_COUNT, chain, &images);

		let future: Box<GpuFuture + Send + Sync> = Box::new(textures_future.join(index_future));

		{
			let mut time = time::Time::default();
			time.set_fixed_time(::std::time::Duration::new(0, 16666666*2));
			world.add_resource(time);
		}

		world.add_resource(Future::new(future));
		world.add_resource(BATCH_CAPACITY as usize);
		world.add_resource(Vector2::new(1024.0f32, 786.0));
		world.add_resource(time::Stopwatch::default());

		let dispatcher = specs::DispatcherBuilder::new()
			.add(SpriteSystem, "sprite", &[])
			.add(buf, "batcher", &["sprite"])
			.build();

		let states = state::StateMachine::new();
		App { world, dispatcher, events_loop, states }
	};

	println!();
	println!("run");

	let arena = arena::Scene { textures };
	app.run(Box::new(arena));
}

#[derive(Clone)]
pub enum Event {
	Frame,
	Fixed,
	W(winit::WindowEvent),
	D(winit::DeviceEvent),
}

struct App<'a, 'b> {
	pub world: specs::World,
	pub dispatcher: specs::Dispatcher<'a, 'b>,
	pub states: state::StateMachine<World, Event>,
	pub events_loop: winit::EventsLoop,
}

impl<'a, 'b> App<'a, 'b> {
	pub fn run(&mut self, state: Box<state::State<World, Event>>) {
		use time::{Time, Stopwatch};

		self.states.initialize(&mut self.world, state);

		self.world.write_resource::<Stopwatch>().start();

		while self.states.is_running() {
			self.advance();
			// XXX: self.world.write_resource::<FrameLimiter>().wait();
			{
				let elapsed = self.world.read_resource::<Stopwatch>().elapsed();
				let mut time = self.world.write_resource::<Time>();
				time.increment_frame_number();
				time.set_delta_time(elapsed);
			}
			let mut stopwatch = self.world.write_resource::<Stopwatch>();
			stopwatch.stop();
			stopwatch.restart();
		}

		::std::process::exit(0)
	}

	fn advance(&mut self) {
		use time::Time;

		/*
		{
			let world = &mut self.world;
			let states = &mut self.states;
			#[cfg(feature = "profiler")]
			profile_scope!("handle_event");

			let events = match world
				.read_resource::<EventChannel<Event>>()
				.lossy_read(&mut self.events_reader_id)
			{
				Ok(data) => data.cloned().collect(),
				_ => Vec::default(),
			};

			for event in events {
				states.handle_event(world, event.clone());
				if !self.ignore_window_close {
					if let &Event::WindowEvent {
						event: WindowEvent::Closed,
						..
					} = &event
					{
						states.stop(world);
					}
				}
			}
		}
		*/
		{
			let states = &mut self.states;
			let world = &mut self.world;
			self.events_loop.poll_events(|event| {
				match event {
					winit::Event::WindowEvent { event, .. } => {
						states.event(world, Event::W(event));
					}
					_ => (),
				}
			});
		}

		{
			let do_fixed = {
				let time = self.world.write_resource::<Time>();
				time.last_fixed_update.elapsed() >= time.fixed_time
			};

			#[cfg(feature = "profiler")] profile_scope!("fixed_update");
			if do_fixed {
				self.states.event(&mut self.world, Event::Fixed);
				self.world.write_resource::<Time>().finish_fixed_update();
			}

			#[cfg(feature = "profiler")] profile_scope!("update");
			self.states.event(&mut self.world, Event::Frame);
		}

		#[cfg(feature = "profiler")] profile_scope!("dispatch");
		self.dispatcher.dispatch(&mut self.world.res);

		/*
		for local in &mut self.locals {
			local.run_now(&self.world.res);
		}
		*/

		#[cfg(feature = "profiler")] profile_scope!("maintain");
		self.world.maintain();

		// TODO: replace this with a more customizable method.
		// TODO: effectively, the user should have more control over error handling here
		// TODO: because right now the app will just exit in case of an error.
		// XXX self.world.write_resource::<Errors>().print_and_exit();
	}
}

/*
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
*/

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
