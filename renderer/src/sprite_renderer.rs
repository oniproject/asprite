use super::*;
use sprite_shader::*;

//vulkano::pipeline::vertex::SingleBufferDefinition<vertex::Vertex>:
//vulkano::pipeline::vertex::VertexSource<vulkano::buffer::cpu_pool::CpuBufferPoolChunk<vertex::Vertex, A>>

pub struct SpriteRenderer {
	pub vbo: VBO<sprite_shader::Vertex>,
	ibo: QuadIBO<u16>,
	pipeline: ArcPipeline<sprite_shader::Vertex>,
	uniform: CpuBufferPool<Uniform>,
	proj_set: DescSet,

	pub fb: Fb,
}

impl SpriteRenderer {
	pub fn new(queue: Arc<Queue>, ibo: QuadIBO<u16>, swapchain: Arc<Swapchain>, images: &[Arc<SwapchainImage>], capacity: usize, group_size: u32) -> Result<Self> {
		let device = queue.device().clone();
		assert_eq!(group_size, 16);
		let shader = Shader::load(device.clone())?;

		let vs = shader.vert_entry_point();
		let fs = shader.frag_entry_point(group_size);

		let mut fb = Fb::simple(swapchain.clone());
		fb.fill(images);

		let pipeline = GraphicsPipeline::start()
			.vertex_input_single_buffer::<sprite_shader::Vertex>()
			.vertex_shader(vs.0, vs.1)
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fs.0, fs.1)
			.blend_alpha_blending()
			.render_pass(Subpass::from(fb.rp.clone(), 0).unwrap())
			.build(device.clone())?;

		let pipeline = Arc::new(pipeline);

		let uniform = CpuBufferPool::new(device.clone(), BufferUsage::all());

		let proj_set = {
			let uniform_buffer_subbuffer = uniform.next(Uniform {
				proj: Matrix4::identity().into(),
			})?;
			let set = PersistentDescriptorSet::start(pipeline.clone(), 0)
				.add_buffer(uniform_buffer_subbuffer)?
				.build()?;
			Arc::new(set)
		};

		let vbo = VBO::new(device.clone(), capacity);

		Ok(Self { pipeline, uniform, proj_set, ibo, vbo, fb })
	}

	pub fn refill(&mut self, images: &[Arc<SwapchainImage>]) {
		self.fb.fill(images);
	}

	pub fn flush(&mut self, cb: CmdBuild, state: DynamicState, textures: &[Texture]) -> Result<CmdBuild> {
		if self.vbo.is_empty() {
			return Ok(cb);
		}

		let count = self.vbo.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;
		let ibo = self.ibo.slice(count).expect("failure index buffer slice");
		let vbo: CpuBufferPoolChunk<sprite_shader::Vertex, Arc<StdMemoryPool>> = self.vbo.flush()?;

		let count = sprite_shader::TextureCount { count: textures.len() as u32 };

		let t = textures;
		let set = PersistentDescriptorSet::start(self.pipeline.clone(), 1)
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

		Ok(cb.draw_indexed(self.pipeline.clone(), state, vbo, ibo, set, count)?)
	}

	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		let proj = Affine::projection(wh.x, wh.y).uniform4();
		let uniform_buffer_subbuffer = self.uniform.next(Uniform {
			proj: proj.into(),
		})?;
		let set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
			.add_buffer(uniform_buffer_subbuffer)?
			.build()?;
		self.proj_set = Arc::new(set);
		Ok(())
	}
}
