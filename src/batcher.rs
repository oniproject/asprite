use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};

use vulkano::buffer::immutable::ImmutableBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::{BufferUsage, BufferAccess};

use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::framebuffer::RenderPassAbstract;

use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::pipeline::viewport::Viewport;
use vulkano::framebuffer::Subpass;

use vulkano::device::{Device, Queue};
use vulkano::framebuffer::Framebuffer;
use vulkano::image::SwapchainImage;

use vulkano::sync::GpuFuture;

use cgmath::{Vector2, Matrix4};

use std::sync::Arc;


use specs::{self, Fetch, FetchMut, Entity, Entities, Join, ReadStorage, WriteStorage};
use specs::{Component, DenseVecStorage, FlaggedStorage};
use specs::UnprotectedStorage;

use shader::*;
use sprite::*;
use texture::*;
use quad_indices::*;

const VERTEX_BY_SPRITE: usize = 4;
const INDEX_BY_SPRITE: usize = 6;
pub const BATCH_CAPACITY: usize = 2_000;

type BoxPipelineLayout = Box<PipelineLayoutAbstract + Send + Sync + 'static>;
type Pipeline<Rp> = Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, BoxPipelineLayout, Arc<Rp>>>;
type Index = Arc<ImmutableBuffer<[u16]>>;
type Fb<Rp> = Arc<Framebuffer<Arc<Rp>, ((), Arc<SwapchainImage>)>>;
type Projection = Arc<DescriptorSet + Send + Sync + 'static>;

smallset!(Group[BaseTexture; fs::TEXTURE_COUNT]);

struct Renderer<Rp> {
	vertices: Vec<Vertex>,
	vertex: CpuBufferPool<Vertex>,
	index: Index,
	pipeline: Pipeline<Rp>,
	group: Group,
}

impl<Rp> Renderer<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Arc<Rp>)
		-> (Self, Box<GpuFuture + Send + Sync>)
	{
		let (index, index_future) = ImmutableBuffer::from_iter(
			QuadIndices(0u16, BATCH_CAPACITY * INDEX_BY_SPRITE),
			BufferUsage::index_buffer(),
			queue.clone(),
		).expect("failed to create index buffer");

		let vertex = CpuBufferPool::<Vertex>::vertex_buffer(device.clone());

		let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
		let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

		let pipeline = GraphicsPipeline::start()
			.vertex_input_single_buffer::<Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fs.main_entry_point(), ())
			.blend_alpha_blending()
			.render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
			.build(device.clone())
			.unwrap();

		let pipeline = Arc::new(pipeline);

		let vertices = Vec::with_capacity(BATCH_CAPACITY);
		let group = Group::new();

		(
			Self { index, vertex, pipeline, vertices, group },
			Box::new(index_future)
		)
	}

	#[inline]
	fn render_sprite(&mut self, sprite: &Sprite, state: DynamicState, proj_set: Projection, mut cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
		if self.vertices.len() >= self.vertices.capacity() {
			cb = self.flush(state.clone(), proj_set.clone(), cb);
		}

		if let Some(ref texture) = sprite.texture {
			let id =
				if let Some(id) = self.group.insert(texture.clone()) {
					id as u32
				} else {
					cb = self.flush(state.clone(), proj_set, cb);
					self.group.array.push(texture.clone());
					0
				};
			let mut vtx = sprite.cache.clone();
			for i in 0..4 {
				vtx[i].texture = id;
			}

			for v in &vtx[..] {
				self.vertices.push(*v);
			}
		}
		cb
	}

	#[inline]
	fn flush(&mut self, state: DynamicState, proj_set: Projection, cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
		if self.vertices.len() == 0 {
			return cb;
		}

		let count = self.vertices.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;

		let vertex_buffer = self.vertex.chunk(self.vertices.drain(..)).unwrap();

		let index_buffer = self.index
			.clone()
			.into_buffer_slice()
			.slice(0..count)
			.unwrap();

		let tex_set = {
			let t = &self.group.array;

			PersistentDescriptorSet::start(self.pipeline.clone(), 1)
				.enter_array().unwrap()

				.add_sampled_image(t[ 0].texture.clone(), t[ 0].sampler.clone()).unwrap()
				.add_sampled_image(t[ 1].texture.clone(), t[ 1].sampler.clone()).unwrap()
				.add_sampled_image(t[ 2].texture.clone(), t[ 2].sampler.clone()).unwrap()
				.add_sampled_image(t[ 3].texture.clone(), t[ 3].sampler.clone()).unwrap()
				.add_sampled_image(t[ 4].texture.clone(), t[ 4].sampler.clone()).unwrap()
				.add_sampled_image(t[ 5].texture.clone(), t[ 5].sampler.clone()).unwrap()
				.add_sampled_image(t[ 6].texture.clone(), t[ 6].sampler.clone()).unwrap()
				.add_sampled_image(t[ 7].texture.clone(), t[ 7].sampler.clone()).unwrap()
				.add_sampled_image(t[ 8].texture.clone(), t[ 8].sampler.clone()).unwrap()
				.add_sampled_image(t[ 9].texture.clone(), t[ 9].sampler.clone()).unwrap()
				//.add_sampled_image(t[10].texture.clone(), t[10].sampler.clone()).unwrap()
				//.add_sampled_image(t[11].texture.clone(), t[11].sampler.clone()).unwrap()

				.leave_array().unwrap()
				.build().unwrap()
		};

		self.group.clear();

		cb.draw_indexed(
			self.pipeline.clone(),
			state,
			vertex_buffer,
			index_buffer,
			(proj_set, Arc::new(tex_set)),
			(),
		).unwrap()
	}
}

pub struct Batcher<Rp> {
	renderer: Renderer<Rp>,
	pub renderpass: Arc<Rp>,

	device: Arc<Device>,
	queue: Arc<Queue>,

	uniform: CpuBufferPool<vs::ty::uni>,
	proj_set: Projection,
}

impl<Rp> Batcher<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Arc<Rp>, proj: Matrix4<f32>)
		-> (Self, Box<GpuFuture + Send + Sync>)
	{
		let (renderer, index_future) = Renderer::new(device.clone(), queue.clone(), renderpass.clone());

		let (index, index_future) = ImmutableBuffer::from_iter(
			QuadIndices(0u16, BATCH_CAPACITY * INDEX_BY_SPRITE),
			BufferUsage::index_buffer(),
			queue.clone(),
		).expect("failed to create index buffer");

		let uniform = CpuBufferPool::<vs::ty::uni>::new(device.clone(), BufferUsage::all());

		let vertex = CpuBufferPool::<Vertex>::vertex_buffer(device.clone());

		let proj_set = {
			let uniform_buffer_subbuffer = {
				let data = vs::ty::uni {
					proj: proj.into(),
				};
				uniform.next(data).unwrap()
			};
			let set = PersistentDescriptorSet::start(renderer.pipeline.clone(), 0)
				.add_buffer(uniform_buffer_subbuffer).unwrap()
				.build().unwrap();
			Arc::new(set)
		};

		(
			Self { device, queue, uniform, renderpass, proj_set, renderer },
			Box::new(index_future)
		)
	}

	pub fn proj_set(&mut self, proj: Matrix4<f32>) {
		let uniform_buffer_subbuffer = {
			let data = vs::ty::uni {
				proj: proj.into(),
			};
			self.uniform.next(data).unwrap()
		};
		let set = PersistentDescriptorSet::start(self.renderer.pipeline.clone(), 0)
			.add_buffer(uniform_buffer_subbuffer).unwrap()
			.build().unwrap();
		self.proj_set = Arc::new(set);
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

	fn run(&mut self, (future, wh, fb, sprites,): Self::SystemData) {
		let wh = *wh;
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
			let proj = self.proj_set.clone();
			cb = self.renderer.render_sprite(&sprite, state.clone(), proj, cb);
		}

		let proj = self.proj_set.clone();
		cb = self.renderer.flush(state, proj, cb);

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
