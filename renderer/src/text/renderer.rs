use vulkano::format::R8Unorm;
use vulkano::image::ImmutableImage;
use vulkano::image::ImageLayout;
use vulkano::image::ImageUsage;

use vulkano::image::swapchain::SwapchainImage;

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::framebuffer::Subpass;
use vulkano::command_buffer::AutoCommandBufferBuilder as CmdBuild;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Queue;

use vulkano::swapchain::Swapchain;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::pipeline::GraphicsPipeline;

use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};
use vulkano::image::Dimensions;

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

pub struct Renderer {
	vbo: VBO<Vertex>,
	ibo: QuadIBO<u16>,

	queue: Arc<Queue>,

	cache: Cache,
	cache_size: (usize, usize),
	cache_pixel_buffer: Vec<u8>,

	sampler: Arc<Sampler>,
	pool: CpuBufferPool<u8>,

	pipeline: ArcPipeline<Vertex>,

	upload: Option<(DescSet, Arc<ImmutableImage<R8Unorm>>)>,

	uniform: CpuBufferPool<Uniform>,
	proj_set: DescSet,

	pub fbo: FBO,
}

impl Renderer {
	pub fn new(queue: Arc<Queue>, ibo: QuadIBO<u16>, swapchain: Arc<Swapchain>, images: &[Arc<SwapchainImage>], width: u32, height: u32) -> Result<Self> {
		let device = queue.device().clone();

		let sampler = Sampler::new(
			device.clone(),
			Filter::Nearest, Filter::Nearest,
			MipmapMode::Nearest,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0, 1.0, 0.0, 0.0)?;

		let pool = CpuBufferPool::upload(device.clone());
		let cache = Cache::new(width, height, 0.1, 0.1);

		let size = width*height;
		let cache_pixel_buffer = vec![0; size as usize];

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

		Ok(Self {
			queue,

			sampler,
			cache, cache_pixel_buffer,
			cache_size: (width as usize, height as usize),
			pool, pipeline,
			uniform,
			proj_set,
			vbo,
			ibo,
			fbo,
			upload: None,
		})
	}

	#[inline]
	pub fn refill(&mut self, images: &[Arc<SwapchainImage>]) {
		self.fbo.fill(images);
	}

	pub fn glyphs<'a>(&mut self, cb: CmdBuild, state: &DynamicState, glyphs: &[PositionedGlyph<'a>], color: [u8; 4], image_num: usize) -> Result<CmdBuild> {
		for g in glyphs.iter().cloned() {
			self.cache.queue_glyph(0, g);
		}

		let (set, _tex, cb) = self.cache_queued(cb)?;
		let set = (self.proj_set.clone(), set);

		{
			let cache = &mut self.cache;
			let iter = glyphs.into_iter()
				.filter_map(|g| cache.rect_for(0, g).unwrap());
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

		let count = self.vbo.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;
		let ibo = self.ibo.slice(count).ok_or_else(|| ErrorKind::NoneError)?;
		let vbo = self.vbo.flush()?;

		let cb = cb.begin_render_pass(self.fbo.at(image_num), false, Vec::new())?;
		let cb = cb.draw_indexed(self.pipeline.clone(), state.clone(), vbo, ibo, set, ())?;
		let cb = cb.end_render_pass()?;
		Ok(cb)
	}

	fn cache_queued(&mut self, cb: CmdBuild) -> Result<(DescSet, Arc<ImmutableImage<R8Unorm>>, CmdBuild)> {
		{
			let upload = &mut self.upload;

			let dst = &mut self.cache_pixel_buffer;
			let stride = self.cache_size.0;

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
		}

		let (tset, tex, cb) = match self.upload {
			Some((ref tset, ref tex)) => (tset.clone(), tex.clone(), cb),
			None => {
				let device = self.queue.device().clone();

				let buffer = self.pool.chunk(self.cache_pixel_buffer.iter().cloned())?;

				let (tex, write) = ImmutableImage::uninitialized(
					device.clone(),
					Dimensions::Dim2d {
						width: self.cache_size.0 as u32,
						height: self.cache_size.1 as u32,
					},
					R8Unorm,
					1,
					ImageUsage {
						sampled: true,
						transfer_destination: true,
						.. ImageUsage::none()
					},
					ImageLayout::General,
					Some(self.queue.family()),
				)?;

				let tset = Arc::new(PersistentDescriptorSet::start(self.pipeline.clone(), 1)
					.add_sampled_image(tex.clone(), self.sampler.clone())?
					.build()?) as DescSet;

				self.upload = Some((tset.clone(), tex.clone()));

				(tset, tex, cb.copy_buffer_to_image(buffer.clone(), write)?)
			}
		};

		Ok((tset, tex, cb))
	}

	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		let proj = Affine::projection(wh.x, wh.y);
		self.proj_set = projection(&self.uniform, self.pipeline.clone(), proj)?;
		Ok(())
	}
}
