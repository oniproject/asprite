error_chain! {
	foreign_links {
		ImageError(::image::ImageError);
		VkImageCreationError(::vulkano::image::ImageCreationError);
		VkSamplerCreationError(::vulkano::sampler::SamplerCreationError);
	}
}
