#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(derive_clone_copy)]
#![feature(const_fn)]

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

extern crate winit;
#[macro_use] extern crate derivative;
#[macro_use] extern crate vulkano;
extern crate vulkano_win;

use vulkano_win::VkSurfaceBuild;
use vulkano::sync::GpuFuture;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::swapchain::{
	Swapchain,
	SurfaceTransform,
	PresentMode,
	SwapchainCreationError,
	acquire_next_image,
	AcquireError,
};
use vulkano::framebuffer::Framebuffer;

use specs::World;

use std::sync::Arc;
use std::time::{Instant, Duration};

use renderer::vertex::*;
use math::*;

mod arena;
mod input;
mod sprite;
mod tsys;
mod state;
mod sprite_batcher;
mod chain;

use chain::*;
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

	let surface = window.surface().clone();

	let queue = physical.queue_families()
		.find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
		.expect("couldn't find a graphical queue family");

	let device_ext = DeviceExtensions {
		khr_swapchain: true,
		.. DeviceExtensions::none()
	};
	let (device, mut queues) = Device::new(
			physical, physical.supported_features(),
			&device_ext, [(queue, 0.5)].iter().cloned())
		.expect("failed to create device");
	let queue = queues.next().unwrap();

	let caps = surface
		.capabilities(physical)
		.expect("failed to get surface capabilities");

	println!("{:?}", caps);

	let dimensions = caps.current_extent.unwrap_or([1024, 768]);
	let format = caps.supported_formats[0].0;

	let renderpass = single_pass_renderpass!(device.clone(),
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: format,
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		).unwrap();
	let renderpass = Arc::new(renderpass);

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

		Texture::load_vec(queue.clone(), device.clone(), &decoded.images).unwrap()
	};

	let mut ticker = Ticker::new();
	let (mut world, mut dispatcher) = {
		let mut world = World::new();
		world.register::<Sprite>();
		world.register::<SpriteTransform>();
		world.register::<arena::Velocity>();

		let arena = arena::Arena::new(textures.clone());
		let input = input::InputSystem { events_loop, add: false };

		let (buf, index_future) = Batcher::new(device.clone(), queue.clone(), renderpass.clone(), BATCH_CAPACITY, TEXTURE_COUNT);

		let future: Box<GpuFuture + Send + Sync> = Box::new(textures_future.join(index_future));
		world.add_resource(future);
		world.add_resource(BATCH_CAPACITY);

		let dispatcher = specs::DispatcherBuilder::new()
			.add(arena, "mark", &[])
			.add(input, "input", &[])
			.add(SpriteSystem, "sprite", &["mark"])
			.add(buf, "batcher", &["sprite"])
			.build();

		(world, dispatcher)
	};

	let mut chain = {
		let usage = caps.supported_usage_flags;
		let alpha = caps.supported_composite_alpha.iter().next().unwrap();

		let (swapchain, images) = Swapchain::new(
				device.clone(), surface, caps.min_image_count,
				format, dimensions, 1,
				usage, &queue, SurfaceTransform::Identity,
				alpha,
				//PresentMode::Immediate,
				PresentMode::Fifo,
				true, None)
			.expect("failed to create swapchain");

		Chain {
			dimensions,
			physical,
			window,
			renderpass,

			images,
			swapchain,

			framebuffers: None,
			recreate_swapchain: false,
		}
	};

	loop {
		world.write_resource::<Box<GpuFuture + Send + Sync>>().cleanup_finished();

		let (fb, image_num, sw_future) = match chain.run() {
			Some(fb) => fb,
			None => continue,
		};

		world.add_resource(fb);
		world.add_resource(chain.dim());

		temporarily_move_out::<Box<GpuFuture + Send + Sync>, _, _>(
			world.write_resource(), |tmp| Box::new(tmp.join(sw_future)));

		ticker.update();
		world.add_resource(ticker.elapsed);

		dispatcher.dispatch(&mut world.res);
		world.maintain();

		temporarily_move_out::<Box<GpuFuture + Send + Sync>, _, _>(
			world.write_resource(), |future| {
				let future = future
					.then_swapchain_present(queue.clone(), chain.swapchain.clone(), image_num)
					.then_signal_fence_and_flush().unwrap();
				Box::new(future)
			});
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
