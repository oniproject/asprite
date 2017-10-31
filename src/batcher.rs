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


use specs::{self, Fetch, Entity, Entities, Join, ReadStorage, WriteStorage};
use specs::{Component, DenseVecStorage, FlaggedStorage};
use specs::UnprotectedStorage;

use shader::*;
use sprite::*;
use texture::*;
use quad_indices::*;

pub const VERTEX_BY_SPRITE: usize = 4;
pub const INDEX_BY_SPRITE: usize = 6;
pub const BATCH_CAPACITY: usize = 2_000;

type BoxPipelineLayout = Box<PipelineLayoutAbstract + Send + Sync + 'static>;
type Fb<Rp> = Arc<Framebuffer<Arc<Rp>, ((), Arc<SwapchainImage>)>>;

smallset!(Group<BaseTexture>[None; 12]);

pub struct Batcher<Rp> {
	pub renderpass: Arc<Rp>,

	device: Arc<Device>,
	queue: Arc<Queue>,

	uniform: CpuBufferPool<vs::ty::uni>,
	proj_set: Arc<DescriptorSet + Send + Sync + 'static>,

	vertex: CpuBufferPool<Vertex>,
	index: Arc<ImmutableBuffer<[u16]>>,

	pipeline: Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, BoxPipelineLayout, Arc<Rp>>>,
	tex_set: Arc<DescriptorSet + Send + Sync + 'static>,

	group: Group,
	vertices: Vec<Vertex>,
}

impl<Rp> Batcher<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Rp, textures: &[BaseTexture], proj: Matrix4<f32>) -> (Self, Box<GpuFuture>) {

		let (index, index_future) = ImmutableBuffer::from_iter(
			QuadIndices(0u16, BATCH_CAPACITY * INDEX_BY_SPRITE),
			BufferUsage::index_buffer(),
			queue.clone(),
		).expect("failed to create index buffer");


		let uniform = CpuBufferPool::<vs::ty::uni>::new(device.clone(), BufferUsage::all());

		let vertex = CpuBufferPool::<Vertex>::vertex_buffer(device.clone());

		let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
		let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

		let renderpass = Arc::new(renderpass);
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

		let tex_set = PersistentDescriptorSet::start(pipeline.clone(), 1)
			.enter_array().unwrap()

			.add_sampled_image(textures[ 0].texture.clone(), textures[ 0].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 1].texture.clone(), textures[ 1].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 2].texture.clone(), textures[ 2].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 3].texture.clone(), textures[ 3].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 4].texture.clone(), textures[ 4].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 5].texture.clone(), textures[ 5].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 6].texture.clone(), textures[ 6].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 7].texture.clone(), textures[ 7].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 8].texture.clone(), textures[ 8].sampler.clone()).unwrap()
			.add_sampled_image(textures[ 9].texture.clone(), textures[ 9].sampler.clone()).unwrap()
			.add_sampled_image(textures[10].texture.clone(), textures[10].sampler.clone()).unwrap()
			.add_sampled_image(textures[11].texture.clone(), textures[11].sampler.clone()).unwrap()

			.leave_array().unwrap()
			.build().unwrap();

		let tex_set = Arc::new(tex_set);
		let proj_set = {
			let uniform_buffer_subbuffer = {
				let data = vs::ty::uni {
					proj: proj.into(),
				};
				uniform.next(data).unwrap()
			};
			let set = PersistentDescriptorSet::start(pipeline.clone(), 0)
				.add_buffer(uniform_buffer_subbuffer).unwrap()
				.build().unwrap();
			Arc::new(set)
		};

		let vertices = Vec::new();
		let group = Group {
			len: 0,
			array: [
				None, None, None, None,
				None, None, None, None,
				None, None, None, None,
			],
		};

		(
			Self { device, queue, uniform, index, vertex, pipeline, renderpass, tex_set, proj_set, vertices, group },
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
		let set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
			.add_buffer(uniform_buffer_subbuffer).unwrap()
			.build().unwrap();
		self.proj_set = Arc::new(set);
	}

	pub fn draw_iterator<I>(&mut self, state: DynamicState, fb: Fb<Rp>, mut iter: I, mut cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder
		where I: Iterator<Item = Vertex> + ExactSizeIterator
	{
		const SIZE: usize = 1000;
		for _ in 0..BATCH_CAPACITY / SIZE {
			self.vertices.extend(iter.by_ref().take(SIZE * VERTEX_BY_SPRITE));
			cb = self.flush(state.clone(), fb.clone(), cb);
			/*
			let vertex_buffer = self.vertex.chunk().unwrap();

			let end = SIZE * VERTEX_BY_SPRITE;

			let index_buffer = self.index
				.clone()
				.into_buffer_slice()
				//.slice(i*SIZE..count*VERTEX_BY_SPRITE)
				.slice(0..end)
				.unwrap();

			cb = cb.draw_indexed(
					self.pipeline.clone(),
					state.clone(),
					vertex_buffer.clone(),
					index_buffer,
					(self.proj_set.clone(), self.tex_set.clone()),
					(),
				).unwrap();
				*/
		}
		cb
	}

	fn flush(&mut self, state: DynamicState, fb: Fb<Rp>, cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
		// create texture mapping from group

		let count = self.vertices.len();

		let vertex_buffer = self.vertex.chunk(self.vertices.drain(..)).unwrap();

		let index_buffer = self.index
			.clone()
			.into_buffer_slice()
			.slice(0..count)
			.unwrap();

		cb.draw_indexed(
			self.pipeline.clone(),
			state,
			vertex_buffer,
			index_buffer,
			(self.proj_set.clone(), self.tex_set.clone()),
			(),
		).unwrap()
	}
}

impl<'a, Rp> specs::System<'a> for Batcher<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	type SystemData = (
		Fetch<'a, Vector2<f32>>,
		Fetch<'a, Fb<Rp>>,
		ReadStorage<'a, Sprite>,
	);

	fn run(&mut self, (wh, qq, sprites,): Self::SystemData) {
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

		for (sprite,) in (&sprites,).join() {
			let texture = match sprite.texture {
				Some(ref texture) => texture,
				None => continue,
			};

			// if we can push to current group - ok
			// another - flush and retry


			for v in &sprite.cache[..] {
				self.vertices.push(*v);
			}
		}
	}
}