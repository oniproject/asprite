use vulkano::device::Queue;
use vulkano::sync::GpuFuture;
use vulkano::swapchain::PresentMode;

use std::collections::VecDeque;
use std::sync::Arc;

use specs::*;

use super::*;

use graphics::*;

use math::*;
use app::Bundle;
use sprite::{Sprite, SpriteSystem, TransformSystem, Local, Global};
use renderer::*;

fn base_font() -> Font<'static> {
	let font = include_bytes!("../../res/FiraSans-Regular.ttf");
	FontCollection::from_bytes(font as &[u8]).into_font().unwrap()
}

pub struct BatcherBundle<'a> {
	pub future: Future,
	pub batcher: Batcher<'a>,
}

impl<'a> BatcherBundle<'a> {
	pub fn new() -> (winit::EventsLoop, Self) {
		let (batcher, index_future, events_loop) = Batcher::new(BATCH_CAPACITY, TEXTURE_COUNT);

		let future = Future::new(index_future);
		(events_loop, Self { future, batcher })
	}
}

impl<'a, 'b> Bundle<'a, 'b> for BatcherBundle<'a> {
	fn bundle(self, world: &mut World, dispatcher: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
		let Self { future, batcher } = self;

		world.register::<Sprite>();

		world.register::<tsys::Parent>();
		world.register::<Global>();
		world.register::<Local>();


		let font = base_font();
		world.add_resource(Graphics::new(font, 14.0));

		world.add_resource(future);
		world.add_resource(Vector2::new(1024.0f32, 786.0));

		dispatcher
			.add(TransformSystem::default(), "transform", &[])
			.add(SpriteSystem, "sprite", &["transform"])
			.add(batcher, "batcher", &["sprite"])
	}
}

pub struct Batcher<'a> {
	pub renderer: Renderer,
	pub chain: Chain<'a>,
	pub queue: Arc<Queue>,
	pub fd: VecDeque<f32>,
}

impl<'a> Batcher<'a> {
	pub fn new(capacity: usize, group_size: u32)
		-> (Self, Box<GpuFuture + Send + Sync>, winit::EventsLoop)
	{
		let (chain, images, events_loop) = Chain::new(|caps| {
			ChainConfig {
				format: caps.supported_formats[0].0,
				present_mode: PresentMode::Fifo,
			}
		});

		let queue = chain.queue.clone();
		let (renderer, index_future) =
			Renderer::new(
				queue.clone(),
				chain.swapchain.clone(),
				&images,
				capacity, group_size).unwrap();

		let fd = VecDeque::new();
		(
			Self { queue, renderer, chain, fd },
			Box::new(index_future),
			events_loop,
		)
	}
}

impl<'a, 'sys> System<'sys> for Batcher<'a> {
	type SystemData = (
		FetchMut<'sys, Future>,
		FetchMut<'sys, Vector2<f32>>,
		Fetch<'sys, Graphics>,
		Fetch<'sys, Time>,
		ReadStorage<'sys, Sprite>,
	);

	fn running_time(&self) -> RunningTime { RunningTime::Long }

	fn run(&mut self, (mut future, mut wh, graphics, time, sprites): Self::SystemData) {
		#[cfg(feature = "profiler")] profile_scope!("rendering");
		future.cleanup_finished();

		let (image_num, mut cb) = {
			#[cfg(feature = "profiler")] profile_scope!("swap");
			let ren = &mut self.renderer;
			match self.chain.run(|m| ren.refill(m)) {
				Some((num, sw_future)) => {
					future.join(sw_future);
					ren.set_num(num);

					let dim = self.chain.dim();
					ren.resize(dim).unwrap();
					*wh = dim;

					const COLOR: [f32; 4] = [
						0.0115, //0x1C as f32 / 255.0,// * 0.26,
						0.113, //0x5E as f32 / 255.0,// * 0.26,
						0.412, //0xAC as f32 / 255.0,// * 0.26,
						1.0,
					];
					(num, ren.clear(COLOR).unwrap())
				},
				None => return,
			}
		};

		let cb = {
			#[cfg(feature = "profiler")] profile_scope!("vg");

			let time = &time;
			self.fd.push_back(time.delta.seconds);
			while self.fd.len() > 300 {
				self.fd.pop_front();
			}

			let color = [0xFF, 0xFF, 0xFF, 0x11];
			{

				fn draw_grid<T, F: FnMut(T, Vector2<f32>, Vector2<f32>) -> T>(mut t: T, wh: Vector2<f32>, s: isize, mut f: F) -> T {
					let w = wh.x as isize / s + 1;
					let h = wh.y as isize / s + 1;
					for y in -1..h {
						let min = Vector2::new(0.0, y as f32 * s as f32);
						let max = Vector2::new(wh.x, y as f32 * s as f32 + 1.0);
						t = f(t, min, max);
					}
					for x in -1..w {
						let min = Vector2::new(x as f32 * s as f32, 0.0);
						let max = Vector2::new(x as f32 * s as f32 + 1.0, wh.x);
						t = f(t, min, max);
					}
					t
				}
				cb = draw_grid(cb, *wh, 24, |cb, min, max|
					self.renderer.x_quad(cb, min, max, color).unwrap()
				);
				cb = draw_grid(cb, *wh, 24 * 4, |cb, min, max|
					self.renderer.x_quad(cb, min, max, color).unwrap()
				);
			}

			let color = [0x0, 0xFF, 0, 0xFF];

			let mut cb = self.renderer.start_vg(cb).unwrap();
			for (i, ms) in self.fd.iter().cloned().enumerate() {
				let min = Vector2::new(wh.x - 300.0 + i as f32, wh.y);
				let max = Vector2::new(wh.x - 301.0 + i as f32, wh.y - ms * 1000.0);
				cb = self.renderer.x_quad(cb, min, max, color).unwrap();
			}


			self.renderer.end_vg(cb).unwrap()
		};

		let cb = {
			#[cfg(feature = "profiler")] profile_scope!("sprites");
			let mut cb = self.renderer.start_sprites(cb).unwrap();
			{
				for sprite in sprites.join() {
					cb = self.renderer.texture_quad(cb,
						sprite.texture.clone(),
						sprite.color,
						&sprite.pos, &sprite.uv).unwrap();
				}
			}
			self.renderer.end_sprites(cb).unwrap()
		};

		let cb = graphics.run(cb, &mut self.renderer).unwrap();

		{
			#[cfg(feature = "profiler")] profile_scope!("end");
			future.then_execute(self.queue.clone(), cb.build().unwrap());
			future.then_swapchain_present(self.queue.clone(), self.chain.swapchain.clone(), image_num);
			future.then_signal_fence_and_flush();
		}
	}
}
