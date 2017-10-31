error_chain! {
	foreign_links {
		ImageError(::image::ImageError);
		VulkanoImageCreationError(::vulkano::image::ImageCreationError);
		VulkanoSamplerCreationError(::vulkano::sampler::SamplerCreationError);
	}
}