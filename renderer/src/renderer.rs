use super::*;
use math::*;

use rusttype::PositionedGlyph;

pub struct Renderer {
	pub sprite: sprite::Renderer,
	pub text: text::Renderer,
	pub vg: vg::Renderer,

	// for clear
	pub fbo: FBO,
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

		let mut fbo = FBO::clear(swapchain.clone());
		fbo.fill(&images);

		let init = Init { queue: queue.clone(), index, swapchain, images };

		let (sprite, fu) = sprite::Renderer::new(init.clone(), capacity, group_size)?;
		let text = text::Renderer::new(init.clone(), 512, 512)?;
		let vg = vg::Renderer::new(init.clone(), capacity)?;

		let index_future = index_future.join(fu);

		let state = DynamicState {
			line_width: None,
			viewports: None,
			scissors: None,
		};

		let last_wh = Vector2::zero();

		Ok((
			Self { sprite, text, vg, fbo, state, last_wh, queue, num: 0, },
			Box::new(index_future)
		))
	}

	#[inline]
	pub fn set_num(&mut self, num: usize) {
		self.num = num;
	}

	#[inline]
	pub fn refill(&mut self, images: &[Arc<SwapchainImage>]) {
		#[cfg(feature = "profiler")] profile_scope!("refill");
		self.fbo.fill(images);
		self.text.refill(images);
		self.sprite.refill(images);
		self.vg.refill(images);
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
		self.vg.proj_set(wh)?;
		Ok(())
	}

	#[inline]
	pub fn clear(&mut self, color: [f32; 4]) -> Result<CmdBuild> {
		let clear = vec![color.into()];
		let fb = self.fbo.at(self.num);
		let cb = CmdBuild::new(self.queue.device().clone(), self.queue.family())?
			.begin_render_pass(fb.clone(), false, clear)?
			.end_render_pass()?;
		Ok(cb)
	}

	#[inline]
	pub fn start_sprites(&mut self, cb: CmdBuild) -> Result<CmdBuild> {
		let fb = self.sprite.fbo.at(self.num);
		Ok(cb.begin_render_pass(fb, false, Vec::new())?)
	}

	#[inline]
	pub fn end_sprites(&mut self, cb: CmdBuild) -> Result<CmdBuild> {
		let cb = self.sprite.flush(cb, &self.state)?;
		Ok(cb.end_render_pass()?)
	}


	#[inline]
	pub fn start_vg(&mut self, cb: CmdBuild) -> Result<CmdBuild> {
		let fb = self.vg.fbo.at(self.num);
		Ok(cb.begin_render_pass(fb, false, Vec::new())?)
	}

	#[inline]
	pub fn end_vg(&mut self, cb: CmdBuild) -> Result<CmdBuild> {
		let cb = self.vg.flush(cb, &self.state)?;
		Ok(cb.end_render_pass()?)
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
	pub fn glyphs<'a>(&mut self, cb: CmdBuild, text: &[PositionedGlyph<'a>], color: [u8; 4]) -> Result<CmdBuild> {
		Ok(self.text.glyphs(cb, &self.state, text, color, self.num)?)
	}

	#[inline]
	pub fn x_quad(&mut self, cb: CmdBuild, min: Vector2<f32>, max: Vector2<f32>, color: [u8; 4]) -> Result<CmdBuild> {
		self.vg.quad(cb, &self.state, min, max, color)
	}

	#[inline]
	pub fn path(&mut self, cb: CmdBuild, vertices: &[vg::Vertex], indices: &[u16]) -> Result<CmdBuild> {
		self.vg.path(cb, &self.state, vertices, indices)
	}

	#[inline]
	pub fn color_quad(&mut self, cb: CmdBuild, min: Vector2<f32>, max: Vector2<f32>, color: [u8; 4]) -> Result<CmdBuild> {
		self.sprite.color_quad(cb, &self.state, min, max, color)
	}

	#[inline]
	pub fn texture_quad(&mut self, cb: CmdBuild, texture: Texture, color: [u8; 4], pos: &[Vector2<f32>; 4], uv: &[[u16;2]; 4]) -> Result<CmdBuild> {
		self.sprite.texture_quad(cb, &self.state, texture, color, pos, uv)
	}
}
