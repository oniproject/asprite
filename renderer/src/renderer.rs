use super::*;

pub struct Renderer {
	pub sprite: SpriteRenderer,
	pub text: TextRenderer,

	pub group: Group,
	pub empty: Texture,
	pub fb: Fb,
}

impl Renderer {
	pub fn new(queue: Arc<Queue>, swapchain: Arc<Swapchain>, images: &[Arc<SwapchainImage>], capacity: usize, group_size: u32)
		-> Result<(Self, Box<GpuFuture + Send + Sync>)>
	{
		let (index, index_future) = QuadIBO::new(queue.clone(), capacity * INDEX_BY_SPRITE)?;

		let group = Group::new(group_size as usize);
		let (fu, empty) = Texture::one_white_pixel(queue.clone())?;
		let index_future = index_future.join(fu);

		let sprite = SpriteRenderer::new(queue.clone(), index.clone(), swapchain.clone(), &images, capacity, group_size)?;
		let text   =   TextRenderer::new(queue.clone(), index.clone(), swapchain.clone(), &images, 1024, 1024)?;

		let mut fb = Fb::clear(swapchain.clone());
		fb.fill(&images);

		Ok((
			Self { empty, group, sprite, text, fb },
			Box::new(index_future)
		))
	}

	pub fn refill(&mut self, images: &[Arc<SwapchainImage>]) {
		self.fb.fill(images);
		self.text.refill(images);
		self.sprite.refill(images);
	}

	pub fn resize(&mut self, wh: Vector2<f32>) -> Result<()> {
		self.sprite.proj_set(wh)?;
		self.text.proj_set(wh)?;
		Ok(())
	}

	pub fn text<'a>(&mut self, cb: CmdBuild, state: DynamicState, text: &Text<'a>, image_num: usize) -> Result<CmdBuild> {
		Ok(self.text.text(cb, state, &text, image_num)?)
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
		cb: CmdBuild, state: DynamicState,
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
		mut cb: CmdBuild, state: DynamicState,

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
