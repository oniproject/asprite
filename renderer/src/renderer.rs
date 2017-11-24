use super::*;
use math::*;

const EMPTY_TEXTURE_ID: u32 = 666;

pub struct Renderer {
	pub sprite: SpriteRenderer,
	pub text: TextRenderer,

	pub group: Group,
	pub empty: Texture,
	pub fb: Fb,
	pub state: DynamicState,
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

		let state = DynamicState {
			line_width: None,
			viewports: None,
			scissors: None,
		};

		Ok((
			Self { empty, group, sprite, text, fb, state },
			Box::new(index_future)
		))
	}

	pub fn refill(&mut self, images: &[Arc<SwapchainImage>]) {
		#[cfg(feature = "profiler")] profile_scope!("refill");
		self.fb.fill(images);
		self.text.refill(images);
		self.sprite.refill(images);
	}

	pub fn resize(&mut self, wh: Vector2<f32>) -> Result<()> {
		self.state = DynamicState {
			line_width: None,
			viewports: Some(vec![Viewport {
				origin: [0.0, 0.0],
				dimensions: wh.into(),
				depth_range: 0.0 .. 1.0,
			}]),
			scissors: None,
		};
		self.sprite.proj_set(wh)?;
		self.text.proj_set(wh)?;
		Ok(())
	}

	#[inline]
	pub fn text<'a>(&mut self, text: &Text<'a>, image_num: usize) -> Result<AutoCommandBuffer> {
		Ok(self.text.text(self.state.clone(), &text, image_num)?)
	}

	#[inline]
	pub fn flush(&mut self, cb: CmdBuild) -> Result<CmdBuild> {
		#[cfg(feature = "profiler")] profile_scope!("flush");

		let t = &mut self.group.array;
		while t.len() < t.capacity() {
			let first = self.empty.clone();
			t.push(first);
		}

		let cb = self.sprite.flush(cb, self.state.clone(), t);
		t.clear();

		cb
	}

	#[inline]
	pub fn color_quad(&mut self,
		mut cb: CmdBuild,
		min: Vector2<f32>,
		max: Vector2<f32>,
		color: [u8; 4]) -> Result<CmdBuild>
	{
		if self.sprite.vbo.is_full() {
			cb = self.flush(cb)?;
		}

		const UV: [[u16;2];4] = zero_uv();

		let pos = [
			Vector2::new(min.x, min.y),
			Vector2::new(max.x, min.y),
			Vector2::new(max.x, max.y),
			Vector2::new(min.x, max.y),
		];

		for i in 0..4 {
			self.sprite.vbo.push(sprite_shader::Vertex {
				position: pos[i].into(),
				uv: UV[i],
				color: color,
				texture: EMPTY_TEXTURE_ID,
			});
		}

		Ok(cb)
	}

	#[inline]
	pub fn texture_quad(&mut self,
		mut cb: CmdBuild,

		texture: Texture,
		color: [u8; 4],
		pos: &[Vector2<f32>; 4],
		uv: &[[u16;2]; 4]) -> Result<CmdBuild>
	{
		//#[cfg(feature = "profiler")] profile_scope!("texture_quad");

		if self.sprite.vbo.is_full() {
			cb = self.flush(cb)?;
		}

		let id = {
			//#[cfg(feature = "profiler")] profile_scope!("tid");
			match self.group.insert(texture) {
				Ok(id) => id as u32,
				Err(texture) => {
					cb = self.flush(cb)?;
					self.group.push(texture);
					0
				}
			}
		};

		{
			//#[cfg(feature = "profiler")] profile_scope!("push");
			for i in 0..4 {
				self.sprite.vbo.vertices.place_back() <- sprite_shader::Vertex {
					position: pos[i].into(),
					uv: uv[i],
					color: color,
					texture: id,
				};
			}
		}

		Ok(cb)
	}
}
