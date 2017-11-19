use super::*;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::swapchain::SwapchainImage;

pub struct Chain<'a, Rp> {
	pub recreate_swapchain: bool,
	pub dimensions: [u32; 2],
	pub window: vulkano_win::Window,
	pub physical: PhysicalDevice<'a>,
	pub renderpass: Arc<Rp>,
	pub swapchain: Arc<Swapchain>,
	pub images: Vec<Arc<SwapchainImage>>,

	pub framebuffers: Option<Vec<Arc<Framebuffer<Arc<Rp>, ((), Arc<SwapchainImage>)>>>>,
}

impl<'a, Rp> Chain<'a, Rp>
	where Rp: RenderPassAbstract + Send + Sync + 'static
{
	pub fn dim(&mut self) -> Vector2<f32> {
		let w = self.dimensions[0] as f32;
		let h = self.dimensions[1] as f32;
		Vector2::new(w, h)
	}
	pub fn run(&mut self) -> Option<(Arc<Framebuffer<Arc<Rp>, ((), Arc<SwapchainImage>)>>, usize, Box<GpuFuture + Send + Sync>)> {
		if self.recreate_swapchain {
			println!("recreate_swapchain");
			self.dimensions = self.window.surface()
				.capabilities(self.physical)
				.expect("failed to get surface capabilities")
				.current_extent.unwrap_or([1024, 768]);

			let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(self.dimensions) {
				Ok(r) => r,
				Err(SwapchainCreationError::UnsupportedDimensions) => return None,
				Err(err) => panic!("recreate swapchain: {:?}", err)
			};

			self.framebuffers = None;

			std::mem::replace(&mut self.swapchain, new_swapchain);
			std::mem::replace(&mut self.images, new_images);

			self.recreate_swapchain = false;
		}

		if self.framebuffers.is_none() {
			let new = self.images.iter().map(|image| {
				let f = Framebuffer::start(self.renderpass.clone())
						.add(image.clone()).unwrap()
						.build().unwrap();
				Arc::new(f)
			}).collect::<Vec<_>>();
			std::mem::replace(&mut self.framebuffers, Some(new));
		}

		let (image_num, sw_future) = match acquire_next_image(self.swapchain.clone(), None) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				self.recreate_swapchain = true;
				return None;
			}
			Err(err) => panic!("{:?}", err)
		};

		let fb = self.framebuffers.as_ref().unwrap()[image_num].clone();

		Some((fb, image_num, Box::new(sw_future)))
	}
}
