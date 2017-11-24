use super::*;
use math::*;

const EMPTY_TEXTURE_ID: u32 = 666;

use rusttype::PositionedGlyph;

pub struct Renderer {
	pub sprite: SpriteRenderer,
	pub text: TextRenderer,

	pub group: Group,
	pub empty: Texture,
	pub fb: Fb,
	pub state: DynamicState,

	pub queue: Arc<Queue>,

	pub last_wh: Vector2<f32>,

	pub num: usize,
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

		let last_wh = Vector2::zero();

		Ok((
			Self { empty, group, sprite, text, fb, state, last_wh, queue, num: 0, },
			Box::new(index_future)
		))
	}

	pub fn set_num(&mut self, num: usize) {
		self.num = num;
	}

	pub fn refill(&mut self, images: &[Arc<SwapchainImage>]) {
		#[cfg(feature = "profiler")] profile_scope!("refill");
		self.fb.fill(images);
		self.text.refill(images);
		self.sprite.refill(images);
	}

	#[inline]
	pub fn resize(&mut self, wh: Vector2<f32>) -> Result<()> {
		if self.last_wh == wh {
			return Ok(());
		}
		self.last_wh = wh;

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
	pub fn text_lay<'a, 'font, 'text>(&mut self, font: &'font Font<'a>, size: f32, text: &'text str, x: f32, y: f32)
		-> ::rusttype::LayoutIter<'font, 'text>
		where 'text: 'font,
	{
		let size = ::rusttype::Scale::uniform(size);
		let pos = ::rusttype::point(x, y);
		font.layout(text, size, pos)
	}

	#[inline]
	pub fn clear(&mut self) -> Result<CmdBuild> {
		let clear = vec![[0.0, 0.0, 1.0, 1.0].into()];
		let fb = self.fb.at(self.num);
		let cb = CmdBuild::new(self.queue.device().clone(), self.queue.family())?
			.begin_render_pass(fb.clone(), false, clear)?
			.end_render_pass()?;
		Ok(cb)
	}

	#[inline]
	pub fn start(&mut self, cb: CmdBuild) -> Result<CmdBuild> {
		let fb = self.sprite.fb.at(self.num);
		Ok(cb.begin_render_pass(fb, false, Vec::new())?)
	}

	#[inline]
	pub fn end(&mut self, cb: CmdBuild) -> Result<CmdBuild> {
		Ok(self.flush(cb)?.end_render_pass()?)
	}

	#[inline]
	pub fn glyphs<'a>(&mut self, cb: CmdBuild, text: &[PositionedGlyph<'a>], color: [u8; 4]) -> Result<CmdBuild> {
		Ok(self.text.glyphs(cb, self.state.clone(), text, color, self.num)?)
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
			self.sprite.vbo.vertices.place_back() <- sprite::Vertex {
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
				self.sprite.vbo.vertices.place_back() <- sprite::Vertex {
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
