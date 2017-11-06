#![feature(const_fn)]
#![feature(conservative_impl_trait)]
#![feature(try_trait)]

#[macro_use] extern crate vulkano;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate derivative;
extern crate cgmath;
extern crate image;

extern crate unicode_normalization;
extern crate rusttype;

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
use vulkano::descriptor::descriptor_set::{DescriptorSet, DescriptorSetDesc, PersistentDescriptorSet, DescriptorSetsCollection};
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
//mod text;

mod sprite_shader;
mod sprite_renderer;

use self::sprite_renderer::*;

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

struct VBO<T> {
	vertices: Vec<T>,
	vertex: CpuBufferPool<T>,
}

pub struct Renderer<Rp> {
	vertices: Vec<Vertex>,
	vertex: CpuBufferPool<Vertex>,

	index: Index,
	group: Group,
	white: Texture,

	sprite: SpriteRenderer<Rp>,
}

impl<Rp> Renderer<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Arc<Rp>, capacity: usize, group_size: u32)
		-> Result<(Self, Box<GpuFuture + Send + Sync>)>
	{
		let (index, index_future) = ImmutableBuffer::from_iter(
			QuadIndices(0u16, capacity * INDEX_BY_SPRITE),
			BufferUsage::index_buffer(),
			queue.clone(),
		)?;

		let vertex = CpuBufferPool::<Vertex>::vertex_buffer(device.clone());
		let vertices = Vec::with_capacity(capacity);
		let group = Group::new(group_size as usize);
		let (fu, white) = Texture::one_white_pixel(queue.clone(), device.clone())?;
		let index_future = index_future.join(fu);

		let pass = Subpass::from(renderpass.clone(), 0)
			.expect("failure subpass creation");
		let sprite = SpriteRenderer::new(device.clone(), pass, group_size)?;

		Ok((
			Self { white, index, vertex, vertices, group, sprite },
			Box::new(index_future)
		))
	}

	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		self.sprite.proj_set(wh)?;
		Ok(())
	}

	pub fn push(&mut self,
		mut cb: AutoCommandBufferBuilder,
		state: DynamicState,

		texture: Texture,
		color: [u8; 4],
		pos: [Vector2<f32>; 4],
		uv: [[u16;2]; 4],
		) -> Result<AutoCommandBufferBuilder>
	{
		if self.vertices.len() >= self.vertices.capacity() {
			cb = self.flush(state.clone(), cb)?;
		}

		let id = match self.group.insert(texture) {
			Ok(id) => id as u32,
			Err(texture) => {
				cb = self.flush(state.clone(), cb)?;
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
		Ok(cb)
	}

	pub fn flush(&mut self, state: DynamicState, cb: AutoCommandBufferBuilder) -> Result<AutoCommandBufferBuilder> {
		if self.vertices.len() == 0 {
			return Ok(cb);
		}

		let count = self.vertices.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;

		let vbo = self.vertex.chunk(self.vertices.drain(..))?;
		let ibo = self.index.clone()
			.into_buffer_slice()
			.slice(0..count)
			.expect("failure index buffer slice");

		let set = {
			let t = &mut self.group.array;
			while t.len() < t.capacity() {
				let first = self.white.clone();
				t.push(first);
			}

			let set = self.sprite.sets(t)?;
			t.clear();
			set
		};

		let pipe = self.sprite.pipe();

		Ok(cb.draw_indexed(pipe, state, vbo, ibo, set, ())?)
	}

	pub fn rectangle(&mut self,
		cb: AutoCommandBufferBuilder,
		state: DynamicState,
		min: Vector2<f32>,
		max: Vector2<f32>,
		color: [u8; 4]) -> Result<AutoCommandBufferBuilder>
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
