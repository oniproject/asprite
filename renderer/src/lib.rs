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
use vulkano::command_buffer::AutoCommandBufferBuilder as CmdBuild;
use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device, Queue};
use vulkano::sync::GpuFuture;
use vulkano::sync::now as vk_now;
use vulkano::format::Format;

use vulkano::memory::pool::StdMemoryPool;
use vulkano::memory::pool::MemoryPool;

use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet, DescriptorSetsCollection};
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
use cgmath::{SquareMatrix, Zero};

use rusttype::{FontCollection, Font, Scale, point, vector, PositionedGlyph, Rect};

use std::sync::Arc;
use std::borrow::Cow;

static MAIN: [u8; 5] = *b"main\0";

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
			fn next(&mut self) -> Option<Self::Item> {
				def!(@step 0, self, $($name => $format,)*);
				None
			}
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

mod texture;
mod quad_indices;
mod group;
mod vertex;
mod errors;
mod affine;
mod vbo;

mod text_shader;
mod text_renderer;

mod sprite_shader;
mod sprite_renderer;

use self::sprite_renderer::*;
use self::text_renderer::*;
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
type Pipeline<Rp, Vtx> = Arc<GraphicsPipeline<SingleBufferDefinition<Vtx>, BoxPipelineLayout, Arc<Rp>>>;
type Index<T> = Arc<ImmutableBuffer<[T]>>;
type Projection = Arc<DescriptorSet + Send + Sync + 'static>;

type BoxFuture = Box<GpuFuture + Send + Sync>;

pub type ChunkVBO<T> = CpuBufferPoolChunk<T, Arc<StdMemoryPool>>;
pub type ChunkIBO<T> = BufferSlice<[T], Index<T>>;

pub trait RendererSubpass {
	fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()>;

	fn draw_indexed(
		&mut self, cb: CmdBuild, state: DynamicState,
		vbo: ChunkVBO<sprite_shader::Vertex>, ibo: ChunkIBO<u16>, textures: &[Texture]
		) -> Result<CmdBuild>;
}

#[derive(Clone)]
struct Share<Rp> {
	device: Arc<Device>,
	queue: Arc<Queue>,
	index: QuadIBO<u16>,
	pass: Subpass<Arc<Rp>>,
}

pub struct Renderer<'a, Rp> {
	sprite: SpriteRenderer<Rp>,
	text: TextRenderer<Rp>,

	font: Font<'a>,

	group: Group,
	empty: Texture,
}

impl<'a, Rp> Renderer<'a, Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, renderpass: Arc<Rp>, capacity: usize, group_size: u32)
		-> Result<(Self, Box<GpuFuture + Send + Sync>)>
	{
		let (index, index_future) = QuadIBO::new(queue.clone(), capacity * INDEX_BY_SPRITE)?;

		let group = Group::new(group_size as usize);
		let (fu, empty) = Texture::one_white_pixel(queue.clone(), device.clone())?;
		let index_future = index_future.join(fu);

		let font = include_bytes!("../../TerminusTTF-4.46.0.ttf");
		let font = FontCollection::from_bytes(font as &[u8]).into_font().unwrap();

		let pass = Subpass::from(renderpass.clone(), 0)
			.expect("failure subpass creation");

		let sprite = SpriteRenderer::new(device.clone(), index.clone(), pass.clone(), capacity, group_size)?;
		let text = TextRenderer::new(device.clone(), queue.clone(), index.clone(), pass.clone(), 1024, 1024)?;

		Ok((
			Self { empty, group, sprite, text, font },
			Box::new(index_future)
		))
	}

	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		self.sprite.proj_set(wh)?;
		self.text.proj_set(wh)?;
		Ok(())
	}

	pub fn test_text(&mut self, cb: CmdBuild, state: DynamicState) -> Result<CmdBuild> {
		let text = "A japanese poem:
	Feel free to type out some text, and delete it with Backspace. You can also try resizing this window.";
		Ok(self.text.paragraph(cb, state, &self.font, &text)?)
	}

	pub fn flush(&mut self, cb: CmdBuild, state: DynamicState) -> Result<CmdBuild> {
		let t = &mut self.group.array;
		while t.len() < t.capacity() {
			let first = self.empty.clone();
			t.push(first);
		}

		let cb = self.sprite.flush(cb, state, t);
		t.clear();

		cb
	}

	pub fn color_quad(&mut self,
		cb: CmdBuild,
		state: DynamicState,
		min: Vector2<f32>,
		max: Vector2<f32>,
		color: [u8; 4]) -> Result<CmdBuild>
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
		mut cb: CmdBuild,
		state: DynamicState,

		texture: Texture,
		color: [u8; 4],
		pos: [Vector2<f32>; 4],
		uv: [[u16;2]; 4]) -> Result<CmdBuild>
	{
		if self.sprite.vbo.is_full() {
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
			self.sprite.vbo.push(sprite_shader::Vertex {
				position: pos[i].into(),
				uv: uv[i],
				color: color,
				texture: id,
			});
		}
		Ok(cb)
	}
}
