use vulkano::format::R8Unorm;
use vulkano::image::StorageImage;
use vulkano::image::ImageLayout;
use vulkano::command_buffer::synced::*;
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::sys::UnsafeCommandBufferBuilderBufferImageCopy;
use vulkano::command_buffer::sys::UnsafeCommandBufferBuilderImageAspect;

use super::*;
use text_shader::*;

use std::iter;

use rusttype::{FontCollection, Font, Scale, point, vector, PositionedGlyph, Rect};
use rusttype::gpu_cache::Cache;

#[inline(always)]
unsafe fn as_sync_cmd_buf<P>(cb: &mut CmdBuild<P>) -> &mut SyncCommandBufferBuilder<P> {
	::std::mem::transmute::<&mut CmdBuild<P>, &mut SyncCommandBufferBuilder<P>>(cb)
}

pub struct TextRenderer<Rp> {
	vbo: VBO<text_shader::Vertex>,
	ibo: QuadIBO<u16>,

	cache: Cache,
	texture: Arc<StorageImage<R8Unorm>>,
	sampler: Arc<Sampler>,
	pool: CpuBufferPool<u8>,

	pipeline: Pipeline<Rp, text_shader::Vertex>,
	uniform: CpuBufferPool<text_shader::Uniform>,

	upload: bool,

	wh: Vector2<f32>,
}

impl<Rp> TextRenderer<Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, ibo: QuadIBO<u16>, pass: Subpass<Arc<Rp>>, width: u32, height: u32) -> Result<Self> {
		let f = ::std::iter::once(queue.family());
		let texture = StorageImage::new(
			device.clone(),
			Dimensions::Dim2d { width, height },
			R8Unorm, f).unwrap();

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

		let shader = text_shader::Shader::load(device.clone())?;

		let vs = shader.vert_entry_point();
		let fs = shader.frag_entry_point();

		let pipeline = GraphicsPipeline::start()
			.vertex_input_single_buffer::<text_shader::Vertex>()
			.vertex_shader(vs.0, vs.1)
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fs.0, fs.1)
			.blend_alpha_blending()
			.render_pass(pass)
			.build(device.clone())?;

		let pipeline = Arc::new(pipeline);

		let uniform = CpuBufferPool::uniform_buffer(device.clone());
		let wh = Vector2::zero();

		let capacity = 2000;
		let vbo = VBO::new(device.clone(), capacity);

		Ok(Self { texture, sampler, cache, pool, pipeline, uniform, wh, vbo, ibo, upload: false, })
	}

	pub fn paragraph(&mut self, cb: CmdBuild, state: DynamicState, font: &Font, text: &str) -> Result<CmdBuild> {
		let width = 500;
		let glyphs = layout_paragraph(&font, Scale::uniform(24.0), width, &text);
		self.glyphs(cb, state, &glyphs)
	}

	pub fn glyphs<'a>(&mut self, mut cb: CmdBuild, state: DynamicState, glyphs: &[PositionedGlyph<'a>]) -> Result<CmdBuild> {
		for g in glyphs.iter().cloned() {
			self.cache.queue_glyph(0, g);
		}

		self.cache_queued(&mut cb)?;

		let cache = &mut self.cache;
		for (uv, pos) in glyphs.into_iter()
				.filter_map(|g| cache.rect_for(0, g).unwrap()) {
			let uv = [
				pack_uv(uv.min.x, uv.min.y),
				pack_uv(uv.max.x, uv.min.y),
				pack_uv(uv.max.x, uv.max.y),
				pack_uv(uv.min.x, uv.max.y),
			];

			let pos = [
				[pos.min.x as f32, pos.min.y as f32],
				[pos.max.x as f32, pos.min.y as f32],
				[pos.max.x as f32, pos.max.y as f32],
				[pos.min.x as f32, pos.max.y as f32],
			];
			for i in 0..4 {
				self.vbo.push(Vertex {
					uv: uv[i], position: pos[i]
				});
			}
		}

		let count = self.vbo.len() / VERTEX_BY_SPRITE * INDEX_BY_SPRITE;
		let ibo = self.ibo.slice(count).expect("failure index buffer slice");
		let vbo = self.vbo.flush()?;

		let proj = {
			let proj = Affine::projection(self.wh.x, self.wh.y).uniform4();
			let uniform_buffer_subbuffer = self.uniform.next(Uniform {
				proj: proj.into(),
				//color: [0xFF, 0xFF, 0xFF, 0xFF],
				color: [1.0, 1.0, 1.0, 1.0],
			})?;
			let set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
				.add_buffer(uniform_buffer_subbuffer)?
				.build()?;
			set
		};

		let tset = PersistentDescriptorSet::start(self.pipeline.clone(), 1)
			.add_sampled_image(self.texture.clone(), self.sampler.clone())?
			.build()?;

		let set = (Arc::new(proj), Arc::new(tset));

		if !self.upload {
			Ok(cb.draw_indexed(self.pipeline.clone(), state, vbo, ibo, set, ())?)
		} else {
			self.upload = false;
			Ok(cb)
		}
	}

	fn cache_queued(&mut self, cb: &mut CmdBuild) -> Result<()> {
		let pool = &mut self.pool;
		let dst = self.texture.clone();

		let cb = unsafe { as_sync_cmd_buf(cb) };

		let upload = &mut self.upload;

		self.cache.cache_queued(|rect, data| {
			*upload = true;

			if data.is_empty() { return; }

			let dst = dst.clone();
			let src = pool.chunk(data.iter().cloned()).unwrap();

			let offset = rect.min;
			let size = rect.max - rect.min;

			unsafe {
				let copy = UnsafeCommandBufferBuilderBufferImageCopy {
					buffer_offset: 0,
					buffer_row_length: 0,
					buffer_image_height: 0,
					image_aspect: UnsafeCommandBufferBuilderImageAspect {
						color: true,
						depth: false,
						stencil: false,
					},
					image_mip_level: 0,
					image_base_array_layer: 0,
					image_layer_count: 1,
					image_offset: [offset.x as i32, offset.y as i32, 0],
					image_extent: [size.x, size.y, 1],
				};

				if let Err(err) = cb.copy_buffer_to_image(
					src, dst,
					ImageLayout::TransferDstOptimal,
					iter::once(copy),
				) {
					println!("{:?} :: {:?}, {:?} size({:?} for {})", err, rect, offset, size, data.len());
				}
			}
		}).unwrap();
		Ok(())
	}

	pub fn proj_set(&mut self, wh: Vector2<f32>) -> Result<()> {
		self.wh = wh;
		Ok(())
	}
}

/*
pub fn run() {
	let font = include_bytes!("../../TerminusTTF-4.46.0.ttf");
	let font = FontCollection::from_bytes(font as &[u8]).into_font().unwrap();

	let text: String = "A japanese poem:
Feel free to type out some text, and delete it with Backspace. You can also try resizing this window."
.into();

	let (cw, ch) = (512, 512);
	let mut cache = Cache::new(cw, ch, 0.1, 0.1);

	let width = 500;
	let glyphs = layout_paragraph(&font, Scale::uniform(24.0), width, &text);

	for g in &glyphs {
		cache.queue_glyph(0, g.clone());
	}

	/*
	let img = StorageImage::new(
	let buf = CpuBufferPool::upload(device.clone());
	*/

	//let cb = AutoCommandBuffer::new(device.clone(), queue.family()).unwrap()
	cache.cache_queued(|rect, data| {
		println!("cache_queued {:?}", rect);

		/*
		let src = buf.chunk(data).unwrap()

		let offset = rect.min;
		let size = rect.max - rect.min;

		let size = [size.x, size.y, 0];
		let offset = [rect.min.x, rect.min.y, 0];
		cb = cb.copy_buffer_to_image_dimensions(
				src, dst,
				offset, size,
				first_layer, num_layers, mipmap
			).unwrap()
			*/
		//copy_buffer_to_image_dimensions<S, D, Px>(
		//self,
		//source: S,
		//destination: D,
		//offset: [u32; 3],
		//size: [u32; 3],
		//first_layer: u32,
		//num_layers: u32,
		//mipmap: u32
		//) -> Result<Self, CopyBufferImageError> 
	});

	let iter = glyphs.iter()
		.filter_map(|g| cache.rect_for(0, &g).unwrap());

	for (uv, pos) in iter {
		println!("{:?} {:?}", uv, pos);
	}

	/*
	cache.cache_queued(|rect, data| {
		cache_tex.main_level().write(glium::Rect {
			left: rect.min.x,
			bottom: rect.min.y,
			width: rect.width(),
			height: rect.height()
		}, glium::texture::RawImage2d {
			data: Cow::Borrowed(data),
			width: rect.width(),
			height: rect.height(),
			format: glium::texture::ClientFormat::U8
		});
	}).unwrap();
	*/
}
*/

fn layout_paragraph<'a>(font: &'a Font, scale: Scale, width: u32, text: &str) -> Vec<PositionedGlyph<'a>> {
	use unicode_normalization::UnicodeNormalization;

	let mut result = Vec::new();
	let v_metrics = font.v_metrics(scale);
	let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
	let mut caret = point(0.0, v_metrics.ascent);
	let mut last_glyph_id = None;

	for c in text.nfc() {
		if c.is_control() {
			match c {
				'\n' => caret = point(0.0, caret.y + advance_height),
				_ => {}
			}
			continue;
		}
		let base_glyph = match font.glyph(c) {
			Some(glyph) => glyph,
			None => continue,
		};
		if let Some(id) = last_glyph_id.take() {
			caret.x += font.pair_kerning(scale, id, base_glyph.id());
		}
		last_glyph_id = Some(base_glyph.id());
		let mut glyph = base_glyph.scaled(scale).positioned(caret);
		if let Some(bb) = glyph.pixel_bounding_box() {
			if bb.max.x > width as i32 {
				caret = point(0.0, caret.y + advance_height);
				glyph = glyph.into_unpositioned().positioned(caret);
				last_glyph_id = None;
			}
		}
		caret.x += glyph.unpositioned().h_metrics().advance_width;
		result.push(glyph);
	}
	result
}
