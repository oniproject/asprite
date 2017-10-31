#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(derive_clone_copy)]
#![feature(const_fn)]

extern crate specs;
extern crate hibitset;
extern crate fnv;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;

extern crate rayon;
extern crate rand;
extern crate cgmath;
extern crate image;
extern crate winit;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;

use vulkano_win::VkSurfaceBuild;

use vulkano::pipeline::viewport::Viewport;

use vulkano::command_buffer::DynamicState;

use vulkano::sync::GpuFuture;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::device::{Queue, Device, DeviceExtensions};
use vulkano::swapchain::{
	Swapchain,
	SurfaceTransform,
	PresentMode,
	SwapchainCreationError,
	acquire_next_image,
	AcquireError,
};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::framebuffer::Framebuffer;
use vulkano::image::SwapchainImage;
use vulkano::buffer::BufferUsage;

use specs::World;
use cgmath::Vector2;


use std::sync::Arc;
use std::time::{Instant, Duration};

#[macro_use]
mod smallset;

mod errors;

mod d8;
mod quad_indices;
mod arena;
mod sprite;
mod shader;
mod transform;
mod tsys;
mod state;
mod texture;
mod batcher;

use sprite::*;
use batcher::*;
//use errors::*;
use texture::*;

use quad_indices::*;

fn _main() {
	#[inline(always)]
	pub fn pack_uv(x: f32, y: f32) -> [u16; 2] {
		let x = (x * 65535.0) as u16;
		let y = (y * 65535.0) as u16;
		[x, y]
	}

	println!("{:?}", pack_uv(0.0, 0.0));
	println!("{:?}", pack_uv(1.0, 0.0));
	println!("{:?}", pack_uv(1.0, 1.0));
	println!("{:?}", pack_uv(0.0, 1.0));
}

fn main() {
	let extensions = vulkano_win::required_extensions();
	let instance = Instance::new(None, &extensions, &[])
		.expect("failed to create instance");

	let physical = PhysicalDevice::enumerate(&instance)
		.next().expect("no device available");
	println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

	let mut events_loop = winit::EventsLoop::new();
	let window = winit::WindowBuilder::new()
		.build_vk_surface(&events_loop, instance.clone())
		.expect("can't build window");

	let mut dimensions;

	let queue = physical.queue_families()
		.find(|&q| q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false))
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

	let (mut sc, mut images) = {
		let caps = window.surface()
			.capabilities(physical)
			.expect("failed to get surface capabilities");

		dimensions = caps.current_extent.unwrap_or([1024, 768]);
		let usage = caps.supported_usage_flags;
		let alpha = caps.supported_composite_alpha.iter().next().unwrap();
		let format = caps.supported_formats[0].0;

		Swapchain::new(
				device.clone(), window.surface().clone(), caps.min_image_count,
				format, dimensions, 1,
				usage, &queue, SurfaceTransform::Identity,
				alpha,
				PresentMode::Immediate, true, None)
			.expect("failed to create swapchain")
	};

	let (textures_future, textures) = load_images(
		queue.clone(), device.clone(), &[
			"./images/rabbitv3.png",
			"./images/rabbitv3_ash.png",
			"./images/rabbitv3_batman.png",
			"./images/rabbitv3_bb8.png",
			"./images/rabbitv3_frankenstein.png",
			"./images/rabbitv3_neo.png",
			"./images/rabbitv3_sonic.png",
			"./images/rabbitv3_spidey.png",
			"./images/rabbitv3_stormtrooper.png",
			"./images/rabbitv3_superman.png",
			"./images/rabbitv3_tron.png",
			"./images/rabbitv3_wolverine.png",
		]).unwrap();

	let mut framebuffers: Option<Vec<Arc<Framebuffer<_,_>>>> = None;
	let mut recreate_swapchain = false;

	let mut world = World::new();
	world.register::<Sprite>();
	world.register::<arena::Velocity>();

	let w = dimensions[0] as f32;
	let h = dimensions[1] as f32;

	world.add_resource(Vector2::new(w, h));

	let mut arena = arena::Arena::new(textures.clone());
	for _ in 0..BATCH_CAPACITY {
		arena.spawn(&mut world);
	}

	//let tsys = tsys::System

	let mut dispatcher = specs::DispatcherBuilder::new()
		.add(arena, "bunny mark", &[])
		.build();

	let renderpass = single_pass_renderpass!(device.clone(),
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: sc.format(),
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		).unwrap();
	let mut ticker = Ticker::new();

	let proj = transform::Affine::projection(w, h).uniform4();

	let (mut buf, index_future) = Batcher::new(device.clone(), queue.clone(), renderpass, &textures, proj);

	let mut previous_frame_end = Box::new(textures_future.join(index_future)) as Box<GpuFuture>;

	let clear = vec![[0.0, 0.0, 1.0, 1.0].into()];

	loop {
		previous_frame_end.cleanup_finished();

		ticker.update();
		world.add_resource(ticker.elapsed);

		dispatcher.dispatch(&mut world.res);

		if recreate_swapchain {
			println!("recreate_swapchain");
			dimensions = window.surface().capabilities(physical)
				.expect("failed to get surface capabilities")
				.current_extent.unwrap_or([1024, 768]);

			let w = dimensions[0] as f32;
			let h = dimensions[1] as f32;

			world.add_resource(Vector2::new(w, h));

			let proj = transform::Affine::projection(w, h).uniform4();
			buf.proj_set(proj);

			let (new_swapchain, new_images) = match sc.recreate_with_dimension(dimensions) {
				Ok(r) => r,
				Err(SwapchainCreationError::UnsupportedDimensions) => continue,
				Err(err) => panic!("recreate swapchain: {:?}", err)
			};

			framebuffers = None;

			std::mem::replace(&mut sc, new_swapchain);
			std::mem::replace(&mut images, new_images);

			recreate_swapchain = false;
		}

		if framebuffers.is_none() {
			let new = images.iter().map(|image| {
				let f = Framebuffer::start(buf.renderpass.clone())
						.add(image.clone()).unwrap()
						.build().unwrap();
				Arc::new(f)
			}).collect::<Vec<_>>();
			std::mem::replace(&mut framebuffers, Some(new));
		}

		let (image_num, future) = match acquire_next_image(sc.clone(), None) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				recreate_swapchain = true;
				continue;
			},
			Err(err) => panic!("{:?}", err)
		};

		let fb = framebuffers.as_ref().unwrap()[image_num].clone();

		let r: (Arc<Framebuffer<_, ((), Arc<SwapchainImage>)>>, Arc<Device>, Arc<Queue>) =
			(fb.clone(), device.clone(), queue.clone());

		world.add_resource(r);

		//let iter = arena.iter();
		use specs::Join;
		let read = world.read::<Sprite>();
		let iter = read.join();
		
		struct ESI<I: Iterator> {
			iter: I,
			size: usize,
			current: usize,
			cache: [Vertex; 4],
		}

		impl<'a, I: Iterator<Item=&'a Sprite>> Iterator for ESI<I> {
			type Item = Vertex;
			fn next(&mut self) -> Option<Self::Item> {
				let cur = self.current % 4;
				self.current += 1;
				if cur == 0 {
					if let Some(sprite) = self.iter.next() {
						self.cache = sprite.cache;
					} else {
						return None;
					}
				}
				Some(self.cache[cur])
			}
			fn size_hint(&self) -> (usize, Option<usize>) { (self.size, Some(self.size)) }
		}
		impl<'a, I: Iterator<Item=&'a Sprite>> ExactSizeIterator for ESI<I> {}

		//world.add_resource(fb);

		let cb = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
			.unwrap()
			.begin_render_pass(fb.clone(), false, clear.clone()).unwrap();


		let state = DynamicState {
			line_width: None,
			viewports: Some(vec![Viewport {
				origin: [0.0, 0.0],
				dimensions: [dimensions[0] as f32, dimensions[1] as f32],
				depth_range: 0.0 .. 1.0,
			}]),
			scissors: None,
		};

		let cb = buf.draw_iterator(
			state,
			fb.clone(),
			ESI {
				iter: iter, size: 5000 * 4,
				current: 0,
				cache: [Vertex::default(); 4]
			},
			cb,
		);

		let cb = cb.end_render_pass().unwrap()
			.build().unwrap();

		let future = previous_frame_end.join(future)
			.then_execute(queue.clone(), cb).unwrap()
			.then_swapchain_present(queue.clone(), sc.clone(), image_num)
			.then_signal_fence_and_flush().unwrap();
		previous_frame_end = Box::new(future) as Box<_>;

		let mut done = false;
		events_loop.poll_events(|ev| {
			use winit::{Event, WindowEvent};
			//use winit::VirtualKeyCode as VK;
			match ev {
				Event::WindowEvent { event: WindowEvent::Closed, .. } => done = true,
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
				_ => ()
			}
		});
		if done { return; }
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
type Tex = (u32, u32, Arc<()>, Arc<()>);

struct Sp {
	texture: Tex,
	vertex: [Vertex; 4],

	color: [u8; 4],

	uv_cache: [Vector2<f32>; 4],
	pos_cache: [Vector2<f32>; 4],
}

struct Batcher {
	quads: Vec<Vertex>,
	groups: Vec<Group<Tex>>,
	current_group: Group<Tex>,
}

impl Batcher {
	fn new() -> Self {
		Self {
			quads: Vec::new(),
			groups: Vec::new(),
			current_group: Group::new(0),
		}
	}

	fn end(&mut self) {
		self.flush();
	}

	fn push(&mut self, sprite: Sp) {
		if self.quads.len() >= 666 {
			self.flush();
		}

		let tex = match self.current_group.insert(sprite.texture.clone()) {
			Some(tex) => tex as u32,
			None => {
				let end = self.current_group.range.end;
				let mut group = ::std::mem::replace(&mut self.current_group, Group::new(end));
				group.textures[0] = Some(sprite.texture.clone());
				self.groups.push(group);
				0
			}
		};

		self.current_group.range.end += 1;

		#[inline(always)]
		pub fn pack_uv(uv: Vector2<f32>) -> [u16; 2] {
			let Vector2 { x, y } = uv;
			let x = (x * 65535.0) as u16;
			let y = (y * 65535.0) as u16;
			[x, y]
		}

		for i in 0..4 {
			self.quads.push(Vertex {
				position: sprite.pos_cache[i].into(),
				uv: pack_uv(sprite.uv_cache[i]),
				texture: tex,
				color: sprite.color,
			})
		}
	}

	fn flush(&mut self) {
		if self.current_group.range.len() != 0 {
			let group = ::std::mem::replace(&mut self.current_group, Group::new(0));
			self.groups.push(group);
		}

		for g in self.groups.drain(..) {
			// create texture uniform
			// then draw all
		}

		self.quads.clear();
	}
}
use std::ops::Range;

const TEXTURES_BY_GROUP: usize = 12;

struct Group<T> {
	range: Range<usize>,
	len: usize,
	textures: [Option<T>; TEXTURES_BY_GROUP],
}

impl<T: PartialEq> Group<T> {
	fn new(start: usize) -> Self {
		Self {
			range: start..start,
			len: 0,
			textures: [
				None, None, None, None,
				None, None, None, None,
				None, None, None, None,
			],
		}
	}
	#[inline(always)]
	fn len(&self) -> usize {
		self.len
	}

	#[inline(always)]
	fn capacity(&self) -> usize {
		TEXTURES_BY_GROUP
	}

	#[inline(always)]
	fn position(&self, v: &T) -> Option<usize> {
		for i in 0..TEXTURES_BY_GROUP {
			match self.textures[i] {
				Some(ref q) if q == v => return Some(i),
				None => return None,
				_ => (),
			}
		}
		None
	}

	#[inline(always)]
	fn insert(&mut self, v: T) -> Option<usize> {
		let pos = self.position(&v);
		if self.len() != self.capacity() && pos.is_none() {
			self.textures[self.len] = Some(v);
			self.len += 1;
			Some(self.len - 1)
		} else {
			pos
		}
	}
}
*/