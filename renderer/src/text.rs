use vulkano::format::R8Unorm;
use vulkano::image::StorageImage;

use super::*;

use rusttype::{FontCollection, Font, Scale, point, vector, PositionedGlyph, Rect};
use rusttype::gpu_cache::Cache;

struct TextRenderer {
	img: Arc<StorageImage<R8Unorm>>,
	pool: CpuBufferPool<u8>,
}

impl TextRenderer {
	pub fn new(device: Arc<Device>, queue: Arc<Queue>, width: u32, height: u32) -> Self {
		let f = ::std::iter::once(queue.family());
		let img = StorageImage::new(
			device.clone(),
			Dimensions::Dim2d { width, height },
			R8Unorm, f).unwrap();

		let pool = CpuBufferPool::upload(device.clone());

		Self { img, pool }
	}

	pub render_text(&mut self, text: &str) {
	}
}

pub fn run() {
	let font = include_bytes!("../../TerminusTTF-4.46.0.ttf");
	let font = FontCollection::from_bytes(font as &[u8]).into_font().unwrap();
	let mut text: String = "A japanese poem:
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
