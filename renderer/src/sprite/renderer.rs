use super::*;
use math::*;

use super::shader::*;
use super::group::*;

use vulkano::image::swapchain::SwapchainImage;

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::framebuffer::Subpass;
use vulkano::command_buffer::AutoCommandBufferBuilder as CmdBuild;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;

use vulkano::swapchain::Swapchain;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::pipeline::GraphicsPipeline;

use std::sync::Arc;


pub const EMPTY_TEXTURE_ID: u32 = 666;

//vulkano::pipeline::vertex::SingleBufferDefinition<vertex::Vertex>:
//vulkano::pipeline::vertex::VertexSource<vulkano::buffer::cpu_pool::CpuBufferPoolChunk<vertex::Vertex, A>>
//use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;

pub struct Renderer {
	pub vbo: VBO<Vertex>,
	ibo: QuadIBO<u16>,
	pipeline: ArcPipeline<Vertex>,
	uniform: CpuBufferPool<Uniform>,
	proj_set: DescSet,
	pub group: Group,
	pub empty: Texture,

	//pool: FixedSizeDescriptorSetsPool<ArcPipeline<Vertex>>,

	pub fbo: FBO,
}

impl Renderer {
	pub fn new<'a>(init: Init<'a>, capacity: usize, group_size: u32) -> Result<(Self, Box<GpuFuture + Send + Sync>)> {
		let Init { queue, index: ibo, swapchain, images } = init;

		let device = queue.device().clone();
		assert_eq!(group_size, 16);
		let shader = Shader::load(device.clone())?;

		let group = Group::new(group_size as usize);

		let vs = shader.vert_entry_point();
		let fs = shader.frag_entry_point(group_size);

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

		let uniform = CpuBufferPool::new(device.clone(), BufferUsage::all());
		let proj_set = projection(&uniform, pipeline.clone(), Matrix4::identity())?;

		let vbo = VBO::new(device.clone(), capacity);

		let (fu, empty) = Texture::one_white_pixel(queue.clone())?;

		//let pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 1);

		Ok((Self { pipeline, uniform, proj_set, ibo, vbo, fbo, group, empty }, fu))
	}

	#[inline]
	pub fn refill(&mut self, images: &[Arc<SwapchainImage>]) {
		self.fbo.fill(images);
	}

	#[inline]
	pub fn flush(&mut self, cb: CmdBuild, state: &DynamicState) -> Result<CmdBuild> {
		if self.vbo.is_empty() {
			return Ok(cb);
		}

		let t = &mut self.group.array;
		while t.len() < t.capacity() {
			let first = self.empty.clone();
			t.push(first);
		}

		let count = self.vbo.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;
		let ibo = self.ibo.slice(count).ok_or_else(|| ErrorKind::NoneError)?;
		let vbo = self.vbo.flush()?;

		let count = TextureCount { count: t.len() as u32 };

		let set = PersistentDescriptorSet::start(self.pipeline.clone(), 1)
		//let set = self.pool.next()
			.enter_array()?

			.add_sampled_image(t[ 0].texture.clone(), t[ 0].sampler.clone())?
			.add_sampled_image(t[ 1].texture.clone(), t[ 1].sampler.clone())?
			.add_sampled_image(t[ 2].texture.clone(), t[ 2].sampler.clone())?
			.add_sampled_image(t[ 3].texture.clone(), t[ 3].sampler.clone())?

			.add_sampled_image(t[ 4].texture.clone(), t[ 4].sampler.clone())?
			.add_sampled_image(t[ 5].texture.clone(), t[ 5].sampler.clone())?
			.add_sampled_image(t[ 6].texture.clone(), t[ 6].sampler.clone())?
			.add_sampled_image(t[ 7].texture.clone(), t[ 7].sampler.clone())?

			.add_sampled_image(t[ 8].texture.clone(), t[ 8].sampler.clone())?
			.add_sampled_image(t[ 9].texture.clone(), t[ 9].sampler.clone())?
			.add_sampled_image(t[10].texture.clone(), t[10].sampler.clone())?
			.add_sampled_image(t[11].texture.clone(), t[11].sampler.clone())?

			.add_sampled_image(t[12].texture.clone(), t[12].sampler.clone())?
			.add_sampled_image(t[13].texture.clone(), t[13].sampler.clone())?
			.add_sampled_image(t[14].texture.clone(), t[14].sampler.clone())?
			.add_sampled_image(t[15].texture.clone(), t[15].sampler.clone())?

			.leave_array()?
			.build()?;

		let set = (self.proj_set.clone(), set);

		t.clear();

		Ok(cb.draw_indexed(self.pipeline.clone(), state.clone(), vbo, ibo, set, count)?)
	}

	#[inline]
	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		let proj = Affine::projection(wh.x, wh.y);
		self.proj_set = projection(&self.uniform, self.pipeline.clone(), proj)?;
		Ok(())
	}

	#[inline]
	pub fn color_quad(&mut self,
		mut cb: CmdBuild, state: &DynamicState,

		min: Vector2<f32>,
		max: Vector2<f32>,
		color: [u8; 4]) -> Result<CmdBuild>
	{
		if self.vbo.is_full() {
			cb = self.flush(cb, state)?;
		}

		const UV: [[u16;2];4] = zero_uv();

		let pos = [
			Vector2::new(min.x, min.y),
			Vector2::new(max.x, min.y),
			Vector2::new(max.x, max.y),
			Vector2::new(min.x, max.y),
		];

		for i in 0..4 {
			self.vbo.vertices.place_back() <- Vertex {
				position: pos[i].into(),
				uv: UV[i],
				color: color,
				texture: EMPTY_TEXTURE_ID,
			};
		}

		Ok(cb)
	}

	#[inline]
	pub fn texture_quad(&mut self,
		mut cb: CmdBuild, state: &DynamicState,

		texture: Texture,
		color: [u8; 4],
		pos: &[Vector2<f32>; 4],
		uv: &[[u16;2]; 4]) -> Result<CmdBuild>
	{
		if self.vbo.is_full() {
			cb = self.flush(cb, state)?;
		}

		let id = match self.group.insert(texture) {
			Ok(id) => id as u32,
			Err(texture) => {
				cb = self.flush(cb, state)?;
				self.group.push(texture);
				0
			}
		};

		for i in 0..4 {
			self.vbo.vertices.place_back() <- Vertex {
				position: pos[i].into(),
				uv: uv[i],
				color: color,
				texture: id,
			};
		}

		Ok(cb)
	}
}
