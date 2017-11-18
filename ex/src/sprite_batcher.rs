use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::image::SwapchainImage;
use vulkano::sync::GpuFuture;

use math::Vector2;

use std::sync::Arc;

use specs::{self, Fetch, FetchMut, Join, ReadStorage};

type Fb<Rp> = Arc<Framebuffer<Arc<Rp>, ((), Arc<SwapchainImage>)>>;

use sprite::*;
use renderer::*;
use renderer::vertex::*;

fn terminus() -> Font<'static> {
	let font = include_bytes!("../../res/TerminusTTF-4.46.0.ttf");
	FontCollection::from_bytes(font as &[u8]).into_font().unwrap()
}

pub struct Batcher<'a, Rp> {
	renderer: Renderer<Rp>,
	device: Arc<Device>,
	queue: Arc<Queue>,
	last_wh: Vector2<f32>,
	font: Font<'a>,
}

impl<'a, Rp> Batcher<'a, Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Arc<Rp>, capacity: usize, group_size: u32)
		-> (Self, Box<GpuFuture + Send + Sync>)
	{
		let pass = Subpass::from(renderpass.clone(), 0)
			.expect("failure subpass creation");

		let (renderer, index_future) =
			Renderer::new(
				device.clone(),
				queue.clone(),
				pass.clone(),
				capacity, group_size).unwrap();

		let font = terminus();
		(
			Self { device, queue, renderer, last_wh: Vector2::new(0.0, 0.0), font },
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
		FetchMut<'sys, Box<GpuFuture + Send + Sync>>,
		Fetch<'sys, Vector2<f32>>,
		Fetch<'sys, Fb<Rp>>,
		ReadStorage<'sys, Sprite>,
	);

	fn running_time(&self) -> specs::RunningTime { specs::RunningTime::Long }

	fn run(&mut self, (future, wh, fb, sprites,): Self::SystemData) {
		let wh = *wh;
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

		let mut cb = AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
			.unwrap()
			.begin_render_pass(fb.clone(), false, clear).unwrap();

		{
				let text = "A japanese poem:
			Feel free to type out some text, and delete it with Backspace. You can also try resizing this window.";

			let text = Text::new(&self.font, text, 24.0)
				.lay(Vector2::new(0.0, 0.0), 500);

			cb = self.renderer.text(cb, state.clone(), &text).unwrap();
		}

		for (sprite,) in (&sprites,).join() {
			cb = self.renderer.texture_quad(cb, state.clone(),
				sprite.texture.clone(),
				sprite.color,
				sprite.pos, sprite.uv).unwrap();
		}

		cb = self.renderer.flush(cb, state).unwrap();

		let cb = cb
			.end_render_pass().unwrap()
			.build().unwrap();

		let q = self.queue.clone();
		temporarily_move_out(future, |f| Box::new(f.then_execute(q, cb).unwrap()));
	}
}
