use super::*;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::swapchain::Capabilities;
use vulkano::swapchain::Surface;
use vulkano_win::Window;
use vulkano::format::Format;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::device::DeviceOwned;

pub struct Chain<'a> {
	pub recreate_swapchain: bool,
	pub dimensions: [u32; 2],
	pub window: Window,
	pub physical: PhysicalDevice<'a>,
	pub swapchain: Arc<Swapchain>,
	pub queue: Arc<Queue>,
}

impl<'a> Chain<'a> {
	pub fn new(
		caps: Capabilities,
		queue: Arc<Queue>,
		surface: Arc<Surface>,
		window: Window,
		physical: PhysicalDevice<'a>,
		format: Format,
		) -> (Self, Vec<Arc<SwapchainImage>>)
	{
		let usage = caps.supported_usage_flags;
		let alpha = caps.supported_composite_alpha.iter().next().unwrap();
		let dimensions = caps.current_extent.unwrap_or([1024, 768]);

		let (swapchain, images) = Swapchain::new(
				queue.device().clone(), surface, caps.min_image_count,
				format, dimensions, 1,
				usage, &queue, SurfaceTransform::Identity,
				alpha,
				//PresentMode::Immediate,
				PresentMode::Fifo,
				true, None)
			.expect("failed to create swapchain");

		(Self {
			dimensions,
			physical,
			window,
			swapchain,
			queue,

			recreate_swapchain: false,
		}, images)
	}
	pub fn format(&self) -> Format {
		self.swapchain.format()
	}
	pub fn dim(&self) -> Vector2<f32> {
		let w = self.dimensions[0] as f32;
		let h = self.dimensions[1] as f32;
		Vector2::new(w, h)
	}
	pub fn run<F>(&mut self, recreate: F) -> Option<(usize, SwapchainAcquireFuture)>
		where F: FnOnce(&[Arc<SwapchainImage>])
	{
		if self.recreate_swapchain {
			println!("recreate_swapchain");
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

pub struct Fbr {
	pub framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
	pub rp: Arc<RenderPassAbstract + Send + Sync>,
}

impl Fbr {
	pub fn new(rp: Arc<RenderPassAbstract + Send + Sync>) -> Self {
		Self {
			rp,
			framebuffers: Vec::new(),
		}
	}

	pub fn clear(swapchain: Arc<Swapchain>) -> Self {
		let device = swapchain.device().clone();
		let render_pass = Arc::new(single_pass_renderpass!(device,
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: swapchain.format(),
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		).unwrap());

		Self::new(render_pass)
	}

	pub fn simple(swapchain: Arc<Swapchain>) -> Self {
		let device = swapchain.device().clone();
		let render_pass = Arc::new(single_pass_renderpass!(device,
			attachments: {
				color: {
					load: Load,
					store: Store,
					format: swapchain.format(),
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		).unwrap());

		Self::new(render_pass)
	}

	pub fn at(&self, num: usize) -> Arc<FramebufferAbstract + Send + Sync> {
		self.framebuffers[num].clone()
	}

	pub fn fill(&mut self, images: &[Arc<SwapchainImage>]) {
		self.framebuffers.clear();
		let rp = self.rp.clone();
		self.framebuffers.extend(images.iter().cloned().map(move |image|
			Arc::new(Framebuffer::start(rp.clone()).add(image).unwrap().build().unwrap())
				as Arc<FramebufferAbstract + Send + Sync>
		));
	}
}
