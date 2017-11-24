use super::*;
use math::*;

use std::sync::Arc;

use super::shader::*;

use vulkano::image::swapchain::SwapchainImage;

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::framebuffer::Subpass;
use vulkano::command_buffer::AutoCommandBufferBuilder as CmdBuild;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Queue;

use vulkano::swapchain::Swapchain;

use vulkano::pipeline::GraphicsPipeline;

//vulkano::pipeline::vertex::SingleBufferDefinition<vertex::Vertex>:
//vulkano::pipeline::vertex::VertexSource<vulkano::buffer::cpu_pool::CpuBufferPoolChunk<vertex::Vertex, A>>
//use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;

pub struct Renderer {
	pub vbo: VBO<Vertex>,
	pub fbo: FBO,
	ibo: QuadIBO<u16>,
	ibo_pool: VBO<u16>,
	pipeline: ArcPipeline<Vertex>,
	uniform: CpuBufferPool<Uniform>,
	proj_set: DescSet,
}

impl Renderer {
	pub fn new(queue: Arc<Queue>, ibo: QuadIBO<u16>, swapchain: Arc<Swapchain>, images: &[Arc<SwapchainImage>], capacity: usize)
		-> Result<Self>
	{
		let device = queue.device().clone();
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

		let uniform = CpuBufferPool::new(device.clone(), BufferUsage::all());
		let proj_set = projection(&uniform, pipeline.clone(), Matrix4::identity())?;

		let vbo = VBO::new(device.clone(), capacity);
		let ibo_pool = VBO::with_usage(device.clone(), BufferUsage::all(), capacity);

		Ok(Self { pipeline, uniform, proj_set, ibo, vbo, fbo, ibo_pool })
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

		let count = self.vbo.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;
		let ibo = self.ibo.slice(count).ok_or_else(|| ErrorKind::NoneError)?;
		let vbo = self.vbo.flush()?;

		let set = self.proj_set.clone();

		Ok(cb.draw_indexed(self.pipeline.clone(), state.clone(), vbo, ibo, set, ())?)
	}

	#[inline]
	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		let proj = Affine::projection(wh.x, wh.y);
		self.proj_set = projection(&self.uniform, self.pipeline.clone(), proj)?;
		Ok(())
	}

	#[inline]
	pub fn path(&mut self,
		mut cb: CmdBuild, state: &DynamicState,
		vertices: &[Vertex],
		indices: &[u16],
		) -> Result<CmdBuild>
	{
		if !self.vbo.is_empty() {
			cb = self.flush(cb, state)?;
		}

		self.vbo.vertices.extend_from_slice(vertices);
		self.ibo_pool.vertices.extend_from_slice(indices);

		let vbo = self.vbo.flush()?;
		let ibo = self.ibo_pool.flush()?;
		let set = self.proj_set.clone();

		Ok(cb.draw_indexed(self.pipeline.clone(), state.clone(), vbo, ibo, set, ())?)
	}

	#[inline]
	pub fn quad(&mut self,
		mut cb: CmdBuild, state: &DynamicState,
		min: Vector2<f32>, max: Vector2<f32>, color: [u8; 4],
		) -> Result<CmdBuild>
	{
		if self.vbo.is_full() {
			cb = self.flush(cb, state)?;
		}

		let pos = [
			Vector2::new(min.x, min.y),
			Vector2::new(max.x, min.y),
			Vector2::new(max.x, max.y),
			Vector2::new(min.x, max.y),
		];

		for i in 0..4 {
			self.vbo.vertices.place_back() <- Vertex {
				position: pos[i].into(),
				color: color,
			};
		}

		Ok(cb)
	}
}
