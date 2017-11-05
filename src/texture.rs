use std::path::Path;
use std::sync::Arc;

use image;
use vulkano;
use vulkano::device::{Device, Queue};
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};
use vulkano::image::ImmutableImage;
use vulkano::image::Dimensions;
use vulkano::format::R8G8B8A8Srgb;
use vulkano::sync::GpuFuture;

use errors::*;
//use d8::*;

#[derive(Clone)]
pub struct BaseTexture {
	pub texture: Arc<ImmutableImage<R8G8B8A8Srgb>>,
	pub sampler: Arc<Sampler>,
	pub wh: (u32, u32),
}

impl PartialEq for BaseTexture {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.texture, &other.texture) &&
		Arc::ptr_eq(&self.sampler, &other.sampler) &&
		self.wh == other.wh
	}
}

/*
#[derive(Clone)]
pub struct UV {
	pub uv: [[u16; 2]; 4],
}

impl UV {
	pub fn new() -> Self {
		Self {
			uv: [
				[0x0000, 0x0000],
				[0xFFFF, 0x0000],
				[0xFFFF, 0xFFFF],
				[0x0000, 0xFFFF],
			],
		}
	}
}


#[derive(Clone)]
pub struct Texture {
	pub base: BaseTexture,
	pub rotation: Option<D8>,
	pub uv: [u16; 2*4],
}

impl Texture {
	pub fn new(base: BaseTexture) -> Self {
		Self {
			base,
			rotation: None,
			uv: [
				0x0000, 0x0000,
				0xFFFF, 0x0000,
				0xFFFF, 0xFFFF,
				0x0000, 0xFFFF,
			],
		}
	}
}
*/

pub fn one_white_pixel(queue: Arc<Queue>, device: Arc<Device>) ->
	Result<(Box<GpuFuture + Send + Sync>, BaseTexture)>
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
	Ok((future, BaseTexture { wh: (1, 1), texture, sampler }))
}

pub fn load_png<P>(queue: Arc<Queue>, device: Arc<Device>, path: P) ->
	Result<(Box<GpuFuture + Send + Sync>, BaseTexture)>
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

	let sampler = Sampler::new(
		device.clone(),
		Filter::Nearest, Filter::Nearest,
		MipmapMode::Nearest,
		SamplerAddressMode::Repeat,
		SamplerAddressMode::Repeat,
		SamplerAddressMode::Repeat,
		0.0, 1.0, 0.0, 0.0)?;

	let future = Box::new(future) as Box<GpuFuture + Send + Sync>;
	Ok((future, BaseTexture { wh, texture, sampler }))
}


pub fn load_images<P>(queue: Arc<Queue>, device: Arc<Device>, images: &[P]) ->
	Result<(Box<GpuFuture + Send + Sync>, Vec<BaseTexture>)>
	where P: AsRef<Path>
{
	let mut future =
		Box::new(vulkano::sync::now(device.clone()))
		as Box<GpuFuture + Send + Sync>;

	let mut textures = Vec::with_capacity(images.len());
	for m in images {
		let (f, t) = load_png(queue.clone(), device.clone(), m)?;
		future = Box::new(future.join(f));
		textures.push(t);
	}

	Ok((future, textures))
}
