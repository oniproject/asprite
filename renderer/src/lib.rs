#![feature(const_fn)]
#![feature(conservative_impl_trait)]

#[macro_use] extern crate vulkano;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate derivative;
extern crate cgmath;
extern crate image;

extern crate unicode_normalization;
extern crate rusttype;

use vulkano::buffer::immutable::ImmutableBuffer;
use vulkano::buffer::cpu_pool::{CpuBufferPool, CpuBufferPoolChunk};
use vulkano::buffer::{BufferUsage, BufferAccess};
use vulkano::buffer::BufferSlice;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, Queue};
use vulkano::sync::GpuFuture;
use vulkano::sync::now as vk_now;
use vulkano::format::Format;

use vulkano::memory::pool::StdMemoryPool;
use vulkano::memory::pool::MemoryPool;

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

mod vbo;

mod sprite_shader;
mod sprite_renderer;

use self::sprite_renderer::*;
use self::quad_indices::*;
use self::group::*;
use self::vbo::*;

pub use self::errors::*;
pub use self::texture::*;
pub use self::vertex::*;
pub use self::affine::*;


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

const VERTEX_BY_SPRITE: usize = 4;
const INDEX_BY_SPRITE: usize = 6;

type BoxPipelineLayout = Box<PipelineLayoutAbstract + Send + Sync + 'static>;
type Pipeline<Rp> = Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, BoxPipelineLayout, Arc<Rp>>>;
type Index = Arc<ImmutableBuffer<[u16]>>;
type Projection = Arc<DescriptorSet + Send + Sync + 'static>;

pub struct Renderer<Rp> {
	vbo: VBO<Vertex>,

	index: Index,
	group: Group,
	empty: Texture,

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

		let vbo = VBO::new(device.clone(), capacity);

		let group = Group::new(group_size as usize);
		let (fu, empty) = Texture::one_white_pixel(queue.clone(), device.clone())?;
		let index_future = index_future.join(fu);

		let pass = Subpass::from(renderpass.clone(), 0)
			.expect("failure subpass creation");
		let sprite = SpriteRenderer::new(device.clone(), pass, group_size)?;

		Ok((
			Self { vbo, empty, index, group, sprite },
			Box::new(index_future)
		))
	}

	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		self.sprite.proj_set(wh)?;
		Ok(())
	}

	pub fn flush(&mut self, cb: AutoCommandBufferBuilder, state: DynamicState) -> Result<AutoCommandBufferBuilder> {
		if self.vbo.is_empty() {
			return Ok(cb);
		}

		let count = self.vbo.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;
		let vbo: CpuBufferPoolChunk<Vertex, Arc<StdMemoryPool>> = self.vbo.flush()?;

		let ibo = self.index.clone()
			.into_buffer_slice()
			.slice(0..count)
			.expect("failure index buffer slice");

		let t = &mut self.group.array;
		while t.len() < t.capacity() {
			let first = self.empty.clone();
			t.push(first);
		}

		let cb = self.sprite.draw_indexed(cb, state, vbo, ibo, t)?;
		t.clear();

		Ok(cb)
	}

	pub fn color_quad(&mut self,
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
		let texture = self.empty.clone();
		self.texture_quad(cb, state, texture, color, pos, zero_uv())
	}

	pub fn texture_quad(&mut self,
		mut cb: AutoCommandBufferBuilder,
		state: DynamicState,

		texture: Texture,
		color: [u8; 4],
		pos: [Vector2<f32>; 4],
		uv: [[u16;2]; 4]) -> Result<AutoCommandBufferBuilder>
	{
		if self.vbo.is_full() {
			cb = self.flush(cb, state.clone())?;
		}

		let id = match self.group.insert(texture) {
			Ok(id) => id as u32,
			Err(texture) => {
				cb = self.flush(cb, state.clone())?;
				self.group.push(texture);
				0
			}
		};

		for i in 0..4 {
			self.vbo.push(Vertex {
				position: pos[i].into(),
				uv: uv[i],
				color: color,
				texture: id,
			});
		}
		Ok(cb)
	}
}
