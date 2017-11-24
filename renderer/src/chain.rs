use super::*;
use math::*;

use vulkano_win::Window;
use vulkano_win::VkSurfaceBuild;

use vulkano::image::swapchain::SwapchainImage;
use vulkano::swapchain::Capabilities;
use vulkano::format::Format;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::device::DeviceOwned;

use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::device::{Device, DeviceExtensions};

lazy_static! {
	static ref INSTANCE: Arc<Instance> = {
		let extensions = vulkano_win::required_extensions();
		Instance::new(None, &extensions, &[])
			.expect("failed to create instance")
	};
}

pub struct ChainConfig {
	pub format: Format,
	pub present_mode: PresentMode,
}

pub struct Chain<'a> {
	pub recreate_swapchain: bool,
	pub dimensions: [u32; 2],
	pub window: Window,
	pub physical: PhysicalDevice<'a>,
	pub swapchain: Arc<Swapchain>,
	pub queue: Arc<Queue>,
	pub device: Arc<Device>,
}

impl<'a> Chain<'a> {
	pub fn new<F>(fmt: F) -> (Self, Vec<Arc<SwapchainImage>>, winit::EventsLoop)
		where F: FnOnce(&Capabilities) -> ChainConfig
	{
		let physical = PhysicalDevice::enumerate(&INSTANCE)
			.next().expect("no device available");

		println!("Using device: {} (type: {:?})", physical.name(), physical.ty());
		println!();

		let events_loop = winit::EventsLoop::new();
		let window = winit::WindowBuilder::new()
			.build_vk_surface(&events_loop, INSTANCE.clone())
			.expect("can't build window");

		let surface = window.surface().clone();

		let queue = physical.queue_families()
			.find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
			.expect("couldn't find a graphical queue family");

		let device_ext = DeviceExtensions {
			khr_swapchain: true,
			.. DeviceExtensions::none()
		};
		let (device, mut queues) = Device::new(
				physical, physical.supported_features(),
				&device_ext, [(queue, 0.5)].iter().cloned())
			.expect("failed to create device");
		let queue = queues.next().unwrap();

		let caps = surface
			.capabilities(physical)
			.expect("failed to get surface capabilities");

		println!();
		println!("{:?}", caps);
		println!();

		let conf = fmt(&caps);

		let usage = caps.supported_usage_flags;
		let alpha = caps.supported_composite_alpha.iter().next().unwrap();
		let dimensions = caps.current_extent.unwrap_or([1024, 768]);

		let (swapchain, images) = Swapchain::new(
				queue.device().clone(), surface, caps.min_image_count,
				conf.format, dimensions, 1,
				usage, &queue, SurfaceTransform::Identity,
				alpha,
				conf.present_mode,
				true, None)
			.expect("failed to create swapchain");

		(
			Self {
				dimensions,
				physical,
				window,
				swapchain,
				queue,
				device,

				recreate_swapchain: false,
			},
			images, events_loop,
		)
	}

	#[inline]
	pub fn format(&self) -> Format {
		self.swapchain.format()
	}
	#[inline]
	pub fn dim(&self) -> Vector2<f32> {
		let w = self.dimensions[0] as f32;
		let h = self.dimensions[1] as f32;
		Vector2::new(w, h)
	}

	#[inline]
	pub fn run<F>(&mut self, recreate: F) -> Option<(usize, SwapchainAcquireFuture)>
		where F: FnOnce(&[Arc<SwapchainImage>])
	{
		if self.recreate_swapchain {
			#[cfg(feature = "profiler")] profile_scope!("recreate_swapchain");
			self.dimensions = self.window.surface()
				.capabilities(self.physical)
				.expect("failed to get surface capabilities")
				.current_extent.unwrap_or([1024, 768]);

			let (new_swapchain, images) = match self.swapchain.recreate_with_dimension(self.dimensions) {
				Ok(r) => r,
				Err(SwapchainCreationError::UnsupportedDimensions) => return None,
				Err(err) => panic!("recreate swapchain: {:?}", err)
			};

			std::mem::replace(&mut self.swapchain, new_swapchain);
			recreate(&images);
			self.recreate_swapchain = false;
		}

		let ret = match acquire_next_image(self.swapchain.clone(), None) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				self.recreate_swapchain = true;
				return None;
			}
			Err(err) => panic!("{:?}", err)
		};

		Some(ret)
	}
}
