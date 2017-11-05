use vulkano::buffer::immutable::ImmutableBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::{BufferUsage, BufferAccess};
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, Queue};
use vulkano::sync::GpuFuture;
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

use cgmath::{Vector2, Matrix4};
use cgmath::SquareMatrix;

use std::sync::Arc;
use std::borrow::Cow;

use transform::*;
use texture::*;
mod quad_indices;
#[macro_use]
mod smallset;
use self::quad_indices::*;
mod vertex;
pub use self::vertex::*;

macro_rules! def {
	(@step $_idx:expr, $self:expr, ) => {};
	(@step $idx:expr, $self:expr, $name:ident => $format:path, $($_name:ident => $_format:path,)*) => {
		if $self.num == $idx {
			$self.num += 1;
			return Some($crate::vulkano::pipeline::shader::ShaderInterfaceDefEntry {
				location: $idx..$idx+1,
				format: $format,
				name: Some(Cow::Borrowed(stringify!($name))),
			});
		}
		def!(@step $idx + 1, $self, $($_name => $_format,)*)
	};

	// counting
	(@step $idx:expr, ) => { $idx };
	(@step $idx:expr, $name:ident => $format:path, $($_name:ident => $_format:path,)*) => {
		def!(@step $idx + 1, $($_name => $_format,)*)
	};

	($class:ident $iter:ident $( $name:ident => $format:path, )*) => {

		#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
		pub struct $class;
		unsafe impl $crate::vulkano::pipeline::shader::ShaderInterfaceDef for $class {
			type Iter = $iter;
			fn elements(&self) -> $iter {
				$iter { num: 0 }
			}
		}

		#[derive(Debug, Copy, Clone)]
		pub struct $iter {
			num: u16,
		}
		impl Iterator for $iter {
			type Item = $crate::vulkano::pipeline::shader::ShaderInterfaceDefEntry;
			#[inline]
			fn next(&mut self) -> Option<Self::Item> {
				def!(@step 0, self, $($name => $format,)*);
				None
			}
			#[inline]
			fn size_hint(&self) -> (usize, Option<usize>) {
				let len = (
					def!(@step 0, $($name => $format,)*)
					- self.num) as usize;
				(len, Some(len))
			}
		}
		impl ExactSizeIterator for $iter {}
	};
}

pub mod fs;
pub mod vs;

const VERTEX_BY_SPRITE: usize = 4;
const INDEX_BY_SPRITE: usize = 6;
pub const BATCH_CAPACITY: usize = 2_000;

type BoxPipelineLayout = Box<PipelineLayoutAbstract + Send + Sync + 'static>;
type Pipeline<Rp> = Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, BoxPipelineLayout, Arc<Rp>>>;
type Index = Arc<ImmutableBuffer<[u16]>>;
type Projection = Arc<DescriptorSet + Send + Sync + 'static>;

smallset!(Group[BaseTexture; fs::TEXTURE_COUNT]);

pub struct Renderer<Rp> {
	vertices: Vec<Vertex>,
	vertex: CpuBufferPool<Vertex>,

	pipeline: Pipeline<Rp>,

	uniform: CpuBufferPool<vs::ty::uni>,
	proj_set: Projection,
	index: Index,
	group: Group,
	white: BaseTexture,
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

		let (fu, white) = one_white_pixel(queue.clone(), device.clone()).unwrap();

		let index_future = index_future.join(fu);

		let uniform = CpuBufferPool::<vs::ty::uni>::new(device.clone(), BufferUsage::all());

		let proj_set = {
			let uniform_buffer_subbuffer = {
				let data = vs::ty::uni {
					proj: Matrix4::identity().into(),
				};
				uniform.next(data).unwrap()
			};
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

	#[inline]
	pub fn push(&mut self,
		mut cb: AutoCommandBufferBuilder,
		state: DynamicState,

		texture: BaseTexture,
		color: [u8; 4],
		pos: [Vector2<f32>; 4],
		uv: [[u16;2]; 4],
		) -> AutoCommandBufferBuilder
	{
		if self.vertices.len() >= self.vertices.capacity() {
			cb = self.flush(state.clone(), cb);
		}

		let id = match self.group.insert_r(texture) {
			Ok(id) => id as u32,
			Err(texture) => {
				cb = self.flush(state.clone(), cb);
				self.group.array.push(texture);
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

