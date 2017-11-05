use vulkano::buffer::immutable::ImmutableBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::{BufferUsage, BufferAccess};
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, Queue};
use vulkano::sync::GpuFuture;
use vulkano::sync::now as vk_now;
use vulkano::format::Format;

use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange};
use vulkano::descriptor::descriptor::{DescriptorBufferDesc, ShaderStages};
use vulkano::descriptor::descriptor::{DescriptorDesc, DescriptorDescTy};
use vulkano::descriptor::descriptor::{DescriptorImageDesc, DescriptorImageDescDimensions, DescriptorImageDescArray};

use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::shader::SpecializationConstants as SpecConstsTrait;
use vulkano::pipeline::shader::SpecializationMapEntry;
use vulkano::pipeline::shader::ShaderModule;
use vulkano::pipeline::shader::GraphicsEntryPoint;
use vulkano::pipeline::shader::GraphicsShaderType;

use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};
use vulkano::image::ImmutableImage;
use vulkano::image::Dimensions;
use vulkano::format::R8G8B8A8Srgb;

use cgmath::{Vector2, Matrix4};
use cgmath::SquareMatrix;

use std::sync::Arc;
use std::borrow::Cow;

mod texture;
mod quad_indices;
mod group;
mod vertex;
mod errors;
mod affine;
mod shader;

use self::quad_indices::*;
use self::group::*;

pub use self::errors::*;
pub use self::texture::*;
pub use self::vertex::*;
pub use self::affine::*;

const VERTEX_BY_SPRITE: usize = 4;
const INDEX_BY_SPRITE: usize = 6;

type BoxPipelineLayout = Box<PipelineLayoutAbstract + Send + Sync + 'static>;
type Pipeline<Rp> = Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, BoxPipelineLayout, Arc<Rp>>>;
type Index = Arc<ImmutableBuffer<[u16]>>;
type Projection = Arc<DescriptorSet + Send + Sync + 'static>;

pub struct Renderer<Rp> {
	vertices: Vec<Vertex>,
	vertex: CpuBufferPool<Vertex>,

	pipeline: Pipeline<Rp>,

	uniform: CpuBufferPool<shader::Uniform>,
	proj_set: Projection,
	index: Index,
	group: Group,
	white: Texture,
}

impl<Rp> Renderer<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Arc<Rp>, capacity: usize, group_size: u32)
		-> (Self, Box<GpuFuture + Send + Sync>)
	{
		let (index, index_future) = ImmutableBuffer::from_iter(
			QuadIndices(0u16, capacity * INDEX_BY_SPRITE),
			BufferUsage::index_buffer(),
			queue.clone(),
		).expect("failed to create index buffer");

		let vertex = CpuBufferPool::<Vertex>::vertex_buffer(device.clone());

		let shader = shader::Shader::load(device.clone()).expect("failed to create shader module");

		let vs = shader.vert_entry_point();
		let fs = shader.frag_entry_point(group_size);

		let pipeline = GraphicsPipeline::start()
			.vertex_input_single_buffer::<Vertex>()
			.vertex_shader(vs.0, vs.1)
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fs.0, fs.1)
			.blend_alpha_blending()
			.render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
			.build(device.clone())
			.unwrap();

		let pipeline = Arc::new(pipeline);

		let vertices = Vec::with_capacity(capacity);
		let group = Group::new(group_size as usize);

		let (fu, white) = Texture::one_white_pixel(queue.clone(), device.clone()).unwrap();

		let index_future = index_future.join(fu);

		let uniform = CpuBufferPool::new(device.clone(), BufferUsage::all());

		let proj_set = {
			let uniform_buffer_subbuffer = uniform.next(shader::Uniform {
				proj: Matrix4::identity().into(),
			}).unwrap();
			let set = PersistentDescriptorSet::start(pipeline.clone(), 0)
				.add_buffer(uniform_buffer_subbuffer).unwrap()
				.build().unwrap();
			Arc::new(set)
		};

		(
			Self { white, index, vertex, pipeline, vertices, group, proj_set, uniform },
			Box::new(index_future)
		)
	}

	#[inline]
	pub fn proj_set(&mut self, wh: Vector2<f32>) {
		let proj = Affine::projection(wh.x, wh.y).uniform4();
		let uniform_buffer_subbuffer = self.uniform.next(shader::Uniform {
			proj: proj.into(),
		}).unwrap();
		let set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
			.add_buffer(uniform_buffer_subbuffer).unwrap()
			.build().unwrap();
		self.proj_set = Arc::new(set);
	}

	#[inline]
	pub fn push(&mut self,
		mut cb: AutoCommandBufferBuilder,
		state: DynamicState,

		texture: Texture,
		color: [u8; 4],
		pos: [Vector2<f32>; 4],
		uv: [[u16;2]; 4],
		) -> AutoCommandBufferBuilder
	{
		if self.vertices.len() >= self.vertices.capacity() {
			cb = self.flush(state.clone(), cb);
		}

		let id = match self.group.insert(texture) {
			Ok(id) => id as u32,
			Err(texture) => {
				cb = self.flush(state.clone(), cb);
				self.group.push(texture);
				0
			}
		};

		for i in 0..4 {
			self.vertices.push(Vertex {
				position: pos[i].into(),
				uv: uv[i],
				color: color,
				texture: id,
			});
		}
		cb
	}

	#[inline]
	pub fn flush(&mut self, state: DynamicState, cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
		if self.vertices.len() == 0 {
			return cb;
		}

		let count = self.vertices.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;

		let vbo = self.vertex.chunk(self.vertices.drain(..)).unwrap();
		let ibo = self.index.clone()
			.into_buffer_slice()
			.slice(0..count)
			.unwrap();

		let tex_set = {
			let t = &mut self.group.array;
			while t.len() < t.capacity() {
				let first = self.white.clone();
				t.push(first);
			}

			let set = PersistentDescriptorSet::start(self.pipeline.clone(), 1)
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

				.add_sampled_image(t[10].texture.clone(), t[10].sampler.clone()).unwrap()
				.add_sampled_image(t[11].texture.clone(), t[11].sampler.clone()).unwrap()
				.add_sampled_image(t[12].texture.clone(), t[12].sampler.clone()).unwrap()
				.add_sampled_image(t[13].texture.clone(), t[13].sampler.clone()).unwrap()
				.add_sampled_image(t[14].texture.clone(), t[14].sampler.clone()).unwrap()
				.add_sampled_image(t[15].texture.clone(), t[15].sampler.clone()).unwrap()

				.leave_array().unwrap()
				.build().unwrap();

			t.clear();

			Arc::new(set)
		};

		let set = (self.proj_set.clone(), tex_set);
		let pipe = self.pipeline.clone();

		cb.draw_indexed(pipe, state, vbo, ibo, set, ()).unwrap()
	}

	#[inline]
	pub fn rectangle(&mut self,
		cb: AutoCommandBufferBuilder,
		state: DynamicState,
		min: Vector2<f32>,
		max: Vector2<f32>,
		color: [u8; 4]) -> AutoCommandBufferBuilder
	{
		let pos = [
			Vector2::new(min.x, min.y),
			Vector2::new(max.x, min.y),
			Vector2::new(max.x, max.y),
			Vector2::new(min.x, max.y),
		];
		let texture = self.white.clone();
		self.push(cb, state, texture, color, pos, zero_uv())
	}
}
