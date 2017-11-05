use vulkano::framebuffer::RenderPassAbstract;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::pipeline::viewport::Viewport;
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::Framebuffer;
use vulkano::image::SwapchainImage;
use vulkano::sync::GpuFuture;

use cgmath::Vector2;

use std::sync::Arc;

use specs::{self, Fetch, FetchMut, Join, ReadStorage};

type Fb<Rp> = Arc<Framebuffer<Arc<Rp>, ((), Arc<SwapchainImage>)>>;

use sprite::*;
use batcher::*;

pub struct Batcher<Rp> {
	renderer: Renderer<Rp>,
	device: Arc<Device>,
	queue: Arc<Queue>,
	last_wh: Vector2<f32>,
}

impl<Rp> Batcher<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Arc<Rp>, capacity: usize, group_size: u32)
		-> (Self, Box<GpuFuture + Send + Sync>)
	{
		let (renderer, index_future) =
			Renderer::new(
				device.clone(),
				queue.clone(),
				renderpass.clone(),
				capacity, group_size);

		(
			Self { device, queue, renderer, last_wh: Vector2::new(0.0, 0.0) },
			Box::new(index_future)
		)
	}

	#[inline]
	fn proj_set(&mut self, wh: Vector2<f32>) {
		if self.last_wh == wh {
			return;
		}
		self.last_wh = wh;
		self.renderer.proj_set(wh);
	}
}

impl<'a, Rp> specs::System<'a> for Batcher<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	type SystemData = (
		FetchMut<'a, Box<GpuFuture + Send + Sync>>,
		Fetch<'a, Vector2<f32>>,
		Fetch<'a, Fb<Rp>>,
		ReadStorage<'a, Sprite>,
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

		for (sprite,) in (&sprites,).join() {
			cb = self.renderer.push(cb, state.clone(),
				sprite.texture.clone(),
				sprite.color,
				sprite.pos, sprite.uv);
		}

		cb = self.renderer.flush(state, cb);

		let cb = cb
			.end_render_pass().unwrap()
			.build().unwrap();

		let q = self.queue.clone();
		temporarily_move_out(future, |f| Box::new(f.then_execute(q, cb).unwrap()));
	}
}

use std::ops::DerefMut;

/// Defeat borrowchecker
/// https://stackoverflow.com/questions/29570781/temporarily-move-out-of-borrowed-content
#[inline(always)]
pub fn temporarily_move_out<T, D, F>(to: D, f: F)
	where D: DerefMut<Target=T>, F: FnOnce(T) -> T
{
	use std::mem::{forget, uninitialized, replace};
	let mut to = to;
	let tmp = replace(&mut *to, unsafe { uninitialized() });
	let new = f(tmp);
	let uninit = replace(&mut *to, new);
	forget(uninit);
}
