use vulkano::device::DeviceOwned;
use vulkano::image::SwapchainImage;
use vulkano::swapchain::Swapchain;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Framebuffer;

use std::sync::Arc;

pub struct FBO {
	pub framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
	pub rp: Arc<RenderPassAbstract + Send + Sync>,
}

impl FBO {
	#[inline]
	pub fn new(rp: Arc<RenderPassAbstract + Send + Sync>) -> Self {
		Self {
			rp,
			framebuffers: Vec::new(),
		}
	}

	#[inline]
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

	#[inline]
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

	#[inline]
	pub fn at(&self, num: usize) -> Arc<FramebufferAbstract + Send + Sync> {
		self.framebuffers[num].clone()
	}

	#[inline]
	pub fn fill(&mut self, images: &[Arc<SwapchainImage>]) {
		self.framebuffers.clear();
		let rp = self.rp.clone();
		self.framebuffers.extend(images.iter().cloned().map(move |image|
			Arc::new(Framebuffer::start(rp.clone()).add(image).unwrap().build().unwrap())
				as Arc<FramebufferAbstract + Send + Sync>
		));
	}
}
