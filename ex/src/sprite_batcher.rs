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

use specs::{self, FetchMut, Join, ReadStorage};

use sprite::*;
use renderer::*;

fn terminus() -> Font<'static> {
	let font = include_bytes!("../../res/TerminusTTF-4.46.0.ttf");
	FontCollection::from_bytes(font as &[u8]).into_font().unwrap()
}

pub struct Batcher<'a, Rp> {
	renderer: Renderer<Rp>,
	chain: Chain<'a>,
	queue: Arc<Queue>,
	last_wh: Vector2<f32>,
	font: Font<'a>,

	fbr: FbR<Rp>,
}

impl<'a, Rp> Batcher<'a, Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(
		queue: Arc<Queue>,
		renderpass: Arc<Rp>,
		capacity: usize,
		group_size: u32,
		chain: Chain<'a>,
		images: &[Arc<SwapchainImage>],
		)
		-> (Self, Box<GpuFuture + Send + Sync>)
	{
		let pass = Subpass::from(renderpass.clone(), 0)
			.expect("failure subpass creation");

		let (renderer, index_future) =
			Renderer::new(
				queue.clone(),
				pass.clone(),
				capacity, group_size).unwrap();

		let mut fbr = FbR::new(renderpass.clone());
		fbr.fill(&images);

		let font = terminus();
		(
			Self { queue, renderer, last_wh: Vector2::new(0.0, 0.0), font, chain, fbr },
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

impl<'a, 'sys, Rp> specs::System<'sys> for Batcher<'a, Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	type SystemData = (
		FetchMut<'sys, Future>,
		FetchMut<'sys, Vector2<f32>>,
		ReadStorage<'sys, Sprite>,
	);

	fn running_time(&self) -> specs::RunningTime { specs::RunningTime::Long }

	fn run(&mut self, (mut future, mut wh, sprites,): Self::SystemData) {
		future.cleanup_finished();

		let (image_num, sw_future) = {
			let fbr = &mut self.fbr;
			match self.chain.run(|m| fbr.fill(m)) {
				Some(fb) => fb,
				None => return,
			}
		};

		let fb = self.fbr.at(image_num);
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

		{
				let text = "A japanese poem:
			Feel free to type out some text, and delete it with Backspace. You can also try resizing this window.";

			let text = Text::new(&self.font, text, 24.0)
				.lay(Vector2::new(100.0, 200.0), 500);

			cb = self.renderer.text(cb, state.clone(), &text).unwrap();
		}

		future.then_execute(self.queue.clone(), cb.end_render_pass().unwrap().build().unwrap());
		future.then_swapchain_present(self.queue.clone(), self.chain.swapchain.clone(), image_num);
		future.then_signal_fence_and_flush();
	}
}
