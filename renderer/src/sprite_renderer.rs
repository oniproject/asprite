use super::*;

use sprite_shader::*;

//vulkano::pipeline::vertex::SingleBufferDefinition<vertex::Vertex>:
//vulkano::pipeline::vertex::VertexSource<vulkano::buffer::cpu_pool::CpuBufferPoolChunk<vertex::Vertex, A>>
//

type ChunkVBO<T> = CpuBufferPoolChunk<T, Arc<StdMemoryPool>>;
type ChunkIBO<T> = BufferSlice<[T], Arc<ImmutableBuffer<[T]>>>;

pub struct SpriteRenderer<Rp> {
	pipeline: Pipeline<Rp>,
	uniform: CpuBufferPool<Uniform>,
	proj_set: Projection,
}

impl<Rp> SpriteRenderer<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, pass: Subpass<Arc<Rp>>, group_size: u32) -> Result<Self> {
		assert_eq!(group_size, 16);
		let shader = Shader::load(device.clone()).expect("failed to create shader module");

		let vs = shader.vert_entry_point();
		let fs = shader.frag_entry_point(group_size);

		let pipeline = GraphicsPipeline::start()
			.vertex_input_single_buffer::<Vertex>()
			.vertex_shader(vs.0, vs.1)
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fs.0, fs.1)
			.blend_alpha_blending()
			.render_pass(pass)
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

		Ok(Self { pipeline, uniform, proj_set })
	}

	pub fn pipe(&mut self) -> Pipeline<Rp> {
		self.pipeline.clone()
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

	/*
	pub fn sets_x(&mut self, t: &[Texture]) -> Result<()> {//-> Result<impl DescriptorSetsCollection> {
		let set = PersistentDescriptorSet::start(self.pipeline.clone(), 1)
			.enter_array()?;

		for t in t {
			set = set.add_sampled_image(t.texture.clone(), t.sampler.clone())?;
		}

		let set = set.leave_array()?.build()?;

		//Ok((self.proj_set.clone(), Arc::new(set)))
		Ok(())
	}
	*/

	pub fn draw_indexed(
		&mut self, cb: AutoCommandBufferBuilder, state: DynamicState,
		vbo: ChunkVBO<Vertex>, ibo: ChunkIBO<u16>, textures: &[Texture]
		) -> Result<AutoCommandBufferBuilder>
	{
		let count = sprite_shader::TextureCount { count: textures.len() as u32 };
		let set = self.sets(textures).unwrap();
		Ok(cb.draw_indexed(self.pipeline.clone(), state, vbo, ibo, set, count)?)
	}

	pub fn sets(&mut self, t: &[Texture]) -> Result<impl DescriptorSetsCollection> {
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

		Ok((self.proj_set.clone(), set))
	}
}

use vulkano::descriptor::descriptor_set::DescriptorWrite;
fn lay<I>(bid: u32, t: I) -> impl Iterator<Item=DescriptorWrite>
	where I: Iterator<Item=Texture>
{
	t.enumerate().map(move |(i, t)| {
		DescriptorWrite::combined_image_sampler(bid, i as u32, &t.sampler, &t.texture)
	})
}
