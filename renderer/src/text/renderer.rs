
use vulkano::image::swapchain::SwapchainImage;

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::framebuffer::Subpass;
use vulkano::command_buffer::AutoCommandBufferBuilder as CmdBuild;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Queue;
use vulkano::device::Device;

use vulkano::swapchain::Swapchain;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::pipeline::GraphicsPipeline;

use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};

use vulkano::format::R8Unorm;
use vulkano::image::Dimensions;
use vulkano::image::immutable::ImmutableImage;
use vulkano::image::immutable::ImmutableImageInitialization;
use vulkano::image::{ImageLayout, ImageUsage};
use vulkano::image::ImageCreationError;

use vulkano::memory::DeviceMemoryAllocError;

use rusttype::PositionedGlyph;
use rusttype::gpu_cache::Cache;

use super::*;
use math::*;

use super::shader::*;

use std::sync::Arc;

/*
#[inline(always)]
unsafe fn as_sync_cmd_buf<P>(cb: &mut CmdBuild<P>) -> &mut SyncCommandBufferBuilder<P> {
	::std::mem::transmute::<&mut CmdBuild<P>, &mut SyncCommandBufferBuilder<P>>(cb)
}
*/

struct CacheImage {
	buffer: Vec<u8>,
	width: u32,
	height: u32,
	queue: Arc<Queue>,
	device: Arc<Device>,
	sampler: Arc<Sampler>,
}

impl CacheImage {
	fn new(queue: Arc<Queue>, width: u32, height: u32) -> Result<Self> {
		let device = queue.device().clone();
		let sampler = Sampler::new(
			device.clone(),
			Filter::Nearest, Filter::Nearest,
			MipmapMode::Nearest,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0, 1.0, 0.0, 0.0)?;

		let size = width*height;
		Ok(Self {
			sampler,
			queue,
			device,
			width, height,
			buffer: vec![0; size as usize],
		})
	}

	fn buffer(&mut self) -> ::std::result::Result<Arc<CpuAccessibleBuffer<[u8]>>, DeviceMemoryAllocError> {
		CpuAccessibleBuffer::from_iter(
			self.device.clone(),
			BufferUsage::all(),
			self.buffer.iter().cloned()
		)
	}

	fn image(&mut self) -> ::std::result::Result<(Arc<ImmutableImage<R8Unorm>>, ImmutableImageInitialization<R8Unorm>), ImageCreationError> {
		use std::iter;
		ImmutableImage::uninitialized(
			self.device.clone(),
			Dimensions::Dim2d {
				width: self.width,
				height: self.height,
			},
			R8Unorm,
			1,
			ImageUsage {
				sampled: true,
				transfer_destination: true,
				.. ImageUsage::none()
			},
			ImageLayout::General,
			iter::once(self.queue.family()),
		)
	}
}

pub struct Renderer {
	vbo: VBO<Vertex>,
	ibo: QuadIBO<u16>,

	cache: Cache<'static>,
	image: CacheImage,

	pipeline: ArcPipeline<Vertex>,

	upload: Option<DescSet>,

	uniform: CpuBufferPool<Uniform>,
	proj_set: DescSet,

	pub fbo: FBO,
}

impl Renderer {
	pub fn new<'a>(init: Init<'a>, width: u32, height: u32) -> Result<Self> {
		let Init { queue, index: ibo, swapchain, images } = init;

		let device = queue.device().clone();

		//let pool = CpuBufferPool::upload(device.clone());
		let cache = Cache::new(width, height, 0.1, 0.1);

		let shader = Shader::load(device.clone())?;

		let vs = shader.vert_entry_point();
		let fs = shader.frag_entry_point();

		let mut fbo = FBO::simple(swapchain.clone());
		fbo.fill(images);

		let sub = Subpass::from(fbo.rp.clone(), 0).ok_or_else(|| ErrorKind::NoneError)?;

		let pipeline = GraphicsPipeline::start()
			.vertex_input_single_buffer::<Vertex>()
			.vertex_shader(vs.0, vs.1)
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fs.0, fs.1)
			.blend_alpha_blending()
			.render_pass(sub)
			.build(device.clone())?;

		let pipeline = Arc::new(pipeline);

		let uniform = CpuBufferPool::uniform_buffer(device.clone());
		let proj_set = projection(&uniform, pipeline.clone(), Matrix4::identity())?;

		let capacity = 2000;
		let vbo = VBO::new(device.clone(), capacity);

		let image = CacheImage::new(queue.clone(), width, height)?;

		Ok(Self {
			cache,
			image,

			pipeline,
			uniform,
			proj_set,
			vbo,
			ibo,
			fbo,
			upload: None,
		})
	}

	pub fn flush(&mut self, cb: CmdBuild, state: &DynamicState, image_num: usize) -> Result<CmdBuild> {
		let (set, cb) = match self.upload {
			Some(ref tset) => (tset.clone(), cb),
			None => {
				let buffer = self.image.buffer()?;
				let (tex, write) = self.image.image()?;
				let tset = Arc::new(PersistentDescriptorSet::start(self.pipeline.clone(), 1)
					.add_sampled_image(tex, self.image.sampler.clone())?
					.build()?) as DescSet;
				self.upload = Some(tset.clone());
				(tset, cb.copy_buffer_to_image(buffer, write)?)
			}
		};

		let set = (self.proj_set.clone(), set);

		let count = self.vbo.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;
		let ibo = self.ibo.slice(count).ok_or_else(|| ErrorKind::NoneError)?;
		let vbo = self.vbo.flush()?;

		let cb = cb.begin_render_pass(self.fbo.at(image_num), false, Vec::new())?;
		let cb = cb.draw_indexed(self.pipeline.clone(), state.clone(), vbo, ibo, set, ())?;
		let cb = cb.end_render_pass()?;
		Ok(cb)
	}

	pub fn glyphs<'a: 'b, 'b, I>(&mut self, glyphs: I, color: [u8; 4])
		where I: Iterator<Item=&'b PositionedGlyph<'a>>
	{
		let cache = &mut self.cache;
		let iter = glyphs.filter_map(|g| cache.rect_for(0, g).unwrap());
		for (uv, pos) in iter {
			self.vbo.push(Vertex {
				uv: pack_uv(uv.min.x, uv.min.y),
				position: [pos.min.x as f32, pos.min.y as f32],
				color,
			});
			self.vbo.push(Vertex {
				uv: pack_uv(uv.max.x, uv.min.y),
				position: [pos.max.x as f32, pos.min.y as f32],
				color,
			});
			self.vbo.push(Vertex {
				uv: pack_uv(uv.max.x, uv.max.y),
				position: [pos.max.x as f32, pos.max.y as f32],
				color,
			});
			self.vbo.push(Vertex {
				uv: pack_uv(uv.min.x, uv.max.y),
				position: [pos.min.x as f32, pos.max.y as f32],
				color,
			});
		}
	}

	pub fn cache_queued<'a, I>(&mut self, glyphs: I) -> Result<()>
		where I: Iterator<Item=PositionedGlyph<'a>>
	{
		for g in glyphs {
			self.cache.queue_glyph(0, g.standalone());
		}

		let upload = &mut self.upload;

		let dst = &mut self.image.buffer;
		let stride = self.image.width as usize;

		self.cache.cache_queued(|rect, src| {
			*upload = None;

			let w = (rect.max.x - rect.min.x) as usize;
			let h = (rect.max.y - rect.min.y) as usize;
			let mut dst_index = rect.min.y as usize * stride + rect.min.x as usize;
			let mut src_index = 0;

			for _ in 0..h {
				let dst_slice = &mut dst[dst_index..dst_index+w];
				let src_slice = &src[src_index..src_index+w];
				dst_slice.copy_from_slice(src_slice);

				dst_index += stride;
				src_index += w;
			}
		})
		.map_err(|e| ErrorKind::CacheWriteErr(e))?;
		Ok(())
	}
}

impl Ren for Renderer {
	#[inline(always)]
	fn init_framebuffer(&mut self, images: &[Arc<SwapchainImage>]) -> Result<()> {
		self.fbo.fill(images);
		Ok(())
	}

	#[inline(always)]
	fn set_projection(&mut self, proj: Affine<f32>) -> Result<()> {
		self.proj_set = projection(&self.uniform, self.pipeline.clone(), proj)?;
		Ok(())
	}
}
