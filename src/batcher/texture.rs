use std::path::Path;
use image;
use super::*;

#[derive(Clone)]
pub struct Texture {
	pub texture: Arc<ImmutableImage<R8G8B8A8Srgb>>,
	pub sampler: Arc<Sampler>,
	pub wh: (u32, u32),
}

impl PartialEq for Texture {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.texture, &other.texture) &&
		Arc::ptr_eq(&self.sampler, &other.sampler) &&
		self.wh == other.wh
	}
}

impl Texture {
	pub fn one_white_pixel(queue: Arc<Queue>, device: Arc<Device>) ->
		Result<(Box<GpuFuture + Send + Sync>, Self)>
	{
		let pixel = &[[0xFFu8; 4]; 1];

		let (texture, future) = ImmutableImage::from_iter(
			pixel.iter().cloned(),
			Dimensions::Dim2d { width: 1, height: 1 },
			R8G8B8A8Srgb,
			queue)?;

		let sampler = Sampler::new(
			device.clone(),
			Filter::Nearest, Filter::Nearest,
			MipmapMode::Nearest,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0, 1.0, 0.0, 0.0)?;

		let future = Box::new(future) as Box<GpuFuture + Send + Sync>;
		Ok((future, Self { wh: (1, 1), texture, sampler }))
	}

	pub fn load_vec<P>(queue: Arc<Queue>, device: Arc<Device>, images: &[P]) ->
		Result<(Box<GpuFuture + Send + Sync>, Vec<Self>)>
		where P: AsRef<Path>
	{
		let mut future =
			Box::new(vk_now(device.clone()))
			as Box<GpuFuture + Send + Sync>;

		let mut textures = Vec::with_capacity(images.len());
		for m in images {
			let (f, t) = Self::load(queue.clone(), device.clone(), m)?;
			future = Box::new(future.join(f));
			textures.push(t);
		}

		Ok((future, textures))
	}

	pub fn load<P>(queue: Arc<Queue>, device: Arc<Device>, path: P) ->
		Result<(Box<GpuFuture + Send + Sync>, Self)>
		where P: AsRef<Path>
	{
		let sampler = Sampler::new(
			device.clone(),
			Filter::Nearest, Filter::Nearest,
			MipmapMode::Nearest,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0, 1.0, 0.0, 0.0)?;

		Self::load_with_sampler(queue, sampler, path)
	}

	pub fn load_with_sampler<P>(queue: Arc<Queue>, sampler: Arc<Sampler>, path: P) ->
		Result<(Box<GpuFuture + Send + Sync>, Self)>
		where P: AsRef<Path>
	{
		let image = image::open(path)?.to_rgba();

		let wh = image.dimensions();
		let image_data = image.into_raw().clone();

		let (texture, future) = ImmutableImage::from_iter(
			image_data.iter().cloned(),
			Dimensions::Dim2d { width: wh.0, height: wh.1 },
			R8G8B8A8Srgb,
			queue)?;

		let future = Box::new(future) as Box<GpuFuture + Send + Sync>;
		Ok((future, Self { wh, texture, sampler }))
	}
}
