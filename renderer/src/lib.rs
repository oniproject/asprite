#![recursion_limit="128"]

#![feature(const_fn)]
#![feature(conservative_impl_trait)]
#![feature(collection_placement)]
#![feature(placement_in_syntax)]
#![feature(try_trait)]

#[cfg(feature = "profiler")]
#[macro_use] extern crate thread_profiler;
#[macro_use] extern crate lazy_static;

extern crate math;

extern crate lyon;
extern crate smallvec;

#[macro_use] extern crate vulkano;
extern crate winit;
extern crate vulkano_win;

#[macro_use] extern crate error_chain;
#[macro_use] extern crate derivative;
extern crate image;

extern crate unicode_normalization;
extern crate rusttype;

use vulkano::image::swapchain::SwapchainImage;

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::command_buffer::AutoCommandBufferBuilder as CmdBuild;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;
use vulkano::sync::now as vk_now;

use vulkano::swapchain::{
	Swapchain,
	SurfaceTransform,
	PresentMode,
	SwapchainCreationError,
	acquire_next_image,
	AcquireError,
};

use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};

use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::vertex::SingleBufferDefinition;

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
				name: Some($crate::Cow::Borrowed(stringify!($name))),
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

mod renderer;
mod quad_indices;
mod vbo;
mod fbo;

mod texture;

pub mod text;
pub mod sprite;
pub mod vg;

mod future;
mod chain;

mod xbuf;

use self::errors::*;
use self::quad_indices::*;
use self::vbo::*;
use self::fbo::*;

pub use self::chain::*;
pub use self::future::*;
pub use self::texture::*;
pub use self::renderer::*;

const VERTEX_BY_SPRITE: usize = 4;
const INDEX_BY_SPRITE: usize = 6;

type BoxPipelineLayout = Box<PipelineLayoutAbstract + Send + Sync + 'static>;
type ArcRenderPass = Arc<RenderPassAbstract + Send + Sync + 'static>;
type ArcPipeline<Vtx> = Arc<GraphicsPipeline<SingleBufferDefinition<Vtx>, BoxPipelineLayout, ArcRenderPass>>;

type DescSet = Arc<DescriptorSet + Send + Sync + 'static>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Uniform {
	pub proj: [[f32; 4]; 4],
}

#[inline]
fn projection<L, P>(uniform: &CpuBufferPool<Uniform>, pipeline: L, proj: P) -> Result<DescSet>
	where
		L: PipelineLayoutAbstract + Send + Sync + 'static,
		P: Into<[[f32; 4]; 4]> + 'static,
{
	let uniform_buffer_subbuffer = uniform.next(Uniform {
		proj: proj.into(),
	})?;
	let set = PersistentDescriptorSet::start(pipeline, 0)
		.add_buffer(uniform_buffer_subbuffer)?
		.build()?;
	Ok(Arc::new(set))
}

type BoxFuture = Box<GpuFuture + Send + Sync>;

#[inline(always)]
pub const fn zero_uv() -> [[u16; 2]; 4] {
	[
		[0x0000, 0x0000],
		[0xFFFF, 0x0000],
		[0xFFFF, 0xFFFF],
		[0x0000, 0xFFFF],
	]
}

#[inline(always)]
pub fn pack_uv(u: f32, v: f32) -> [u16; 2] {
	let u = (u * 65535.0) as u16;
	let v = (v * 65535.0) as u16;
	[u, v]
}

