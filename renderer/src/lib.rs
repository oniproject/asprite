#![feature(const_fn)]
#![feature(conservative_impl_trait)]

extern crate math;
use math::*;

#[macro_use] extern crate vulkano;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate derivative;
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

pub use rusttype::{FontCollection, Font};

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

pub mod errors;
pub mod vertex;

mod renderer;
mod quad_indices;
mod group;
mod vbo;

mod texture;
mod text;

mod text_shader;
mod text_renderer;

mod sprite_shader;
mod sprite_renderer;

use self::errors::*;
use self::sprite_renderer::*;
use self::text_renderer::*;
use self::quad_indices::*;
use self::group::*;
use self::vbo::*;

use self::vertex::*;

pub use self::text::*;
pub use self::texture::*;
pub use self::renderer::*;

const VERTEX_BY_SPRITE: usize = 4;
const INDEX_BY_SPRITE: usize = 6;

type BoxPipelineLayout = Box<PipelineLayoutAbstract + Send + Sync + 'static>;
type Pipeline<Rp, Vtx> = Arc<GraphicsPipeline<SingleBufferDefinition<Vtx>, BoxPipelineLayout, Arc<Rp>>>;
type Index<T> = Arc<ImmutableBuffer<[T]>>;
type Projection = Arc<DescriptorSet + Send + Sync + 'static>;

type BoxFuture = Box<GpuFuture + Send + Sync>;

//type ChunkVBO<T> = CpuBufferPoolChunk<T, Arc<StdMemoryPool>>;
type ChunkIBO<T> = BufferSlice<[T], Index<T>>;
