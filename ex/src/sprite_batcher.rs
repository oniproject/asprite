use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::SwapchainImage;
use vulkano::sync::GpuFuture;

use math::*;

use std::sync::Arc;

use specs::{self, Entities, Fetch, FetchMut, Join, ReadStorage};

use std::time::Duration;
use arena::duration_to_secs;

use sprite::*;
use renderer::*;

fn terminus() -> Font<'static> {
	let font = include_bytes!("../../res/TerminusTTF-4.46.0.ttf");
	FontCollection::from_bytes(font as &[u8]).into_font().unwrap()
}

pub struct Batcher<'a> {
	renderer: Renderer,
	chain: Chain<'a>,
	queue: Arc<Queue>,
	last_wh: Vector2<f32>,
	font: Font<'a>,
}

impl<'a> Batcher<'a> {
	pub fn new(
		capacity: usize,
		group_size: u32,
		chain: Chain<'a>,
		images: &[Arc<SwapchainImage>],
		)
		-> (Self, Box<GpuFuture + Send + Sync>)
	{
		let queue = chain.queue.clone();
		let (renderer, index_future) =
			Renderer::new(
				queue.clone(),
				chain.swapchain.clone(),
				&images,
				capacity, group_size).unwrap();

		let font = terminus();
		(
			Self { queue, renderer, last_wh: Vector2::new(0.0, 0.0), font, chain },
			Box::new(index_future)
		)
	}

	#[inline]
	fn proj_set(&mut self, wh: Vector2<f32>) {
		if self.last_wh == wh {
			return;
		}
		self.last_wh = wh;
		self.renderer.resize(wh).unwrap();
	}
}

impl<'a, 'sys> specs::System<'sys> for Batcher<'a> {
	type SystemData = (
		FetchMut<'sys, Future>,
		FetchMut<'sys, Vector2<f32>>,
		ReadStorage<'sys, Sprite>,
		Entities<'sys>,
		Fetch<'sys, Duration>,
	);

	fn running_time(&self) -> specs::RunningTime { specs::RunningTime::Long }

	fn run(&mut self, (mut future, mut wh, sprites, e, dt): Self::SystemData) {
		future.cleanup_finished();

		let (image_num, sw_future) = {
			let ren = &mut self.renderer;
			match self.chain.run(|m| ren.refill(m)) {
				Some(v) => v,
				None => return,
			}
		};

		future.join(Box::new(sw_future));

		let wh = {
			let dim = self.chain.dim();
			*wh = dim;
			dim
		};

		self.proj_set(wh);
		let state = DynamicState {
			line_width: None,
			viewports: Some(vec![Viewport {
				origin: [0.0, 0.0],
				dimensions: wh.into(),
				depth_range: 0.0 .. 1.0,
			}]),
			scissors: None,
		};

		let clear = vec![[0.0, 0.0, 1.0, 1.0].into()];

		let fb = self.renderer.fbr.at(image_num);
		let mut cb = AutoCommandBufferBuilder::primary_one_time_submit(self.queue.device().clone(), self.queue.family())
			.unwrap()
			.begin_render_pass(fb.clone(), false, clear).unwrap();

		for (sprite,) in (&sprites,).join() {
			cb = self.renderer.texture_quad(cb, state.clone(),
				sprite.texture.clone(),
				sprite.color,
				sprite.pos, sprite.uv).unwrap();
		}

		cb = self.renderer.flush(cb, state.clone()).unwrap();
		future.then_execute(self.queue.clone(), cb.end_render_pass().unwrap().build().unwrap());

		{
			let dt = duration_to_secs(*dt);
			use specs::Join;
			let text = format!("{} {} A japanese poem: 123 456 7890
				Feel free to type out some text, and delete it with Backspace. You can also try resizing this window.",
			e.join().count(), dt);

			let fb = self.renderer.text.fbr.at(image_num);

			let mut cb = AutoCommandBufferBuilder::primary_one_time_submit(self.queue.device().clone(), self.queue.family())
				.unwrap()
				.begin_render_pass(fb.clone(), false, Vec::new()).unwrap();

			let text = Text::new(&self.font, text, 24.0)
				.lay(Vector2::new(100.0, 200.0), 500);

			cb = self.renderer.text(cb, state.clone(), &text).unwrap();
			future.then_execute(self.queue.clone(), cb.end_render_pass().unwrap().build().unwrap());
		}

		future.then_swapchain_present(self.queue.clone(), self.chain.swapchain.clone(), image_num);
		future.then_signal_fence_and_flush();
	}
}
