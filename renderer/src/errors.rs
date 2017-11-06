error_chain! {
	foreign_links {
		//NoneError(::std::option::NoneError);
		ImageError(::image::ImageError);
		VkImageCreationError(::vulkano::image::ImageCreationError);
		VkSamplerCreationError(::vulkano::sampler::SamplerCreationError);
		VkOomError(::vulkano::OomError);
		VkGraphicsPipelineCreationError(::vulkano::pipeline::GraphicsPipelineCreationError);
		VkDeviceMemoryAllocError(::vulkano::memory::DeviceMemoryAllocError);
		VkPersistentDescriptorSetError(::vulkano::descriptor::descriptor_set::PersistentDescriptorSetError);
		VkPersistentDescriptorSetBuildError(::vulkano::descriptor::descriptor_set::PersistentDescriptorSetBuildError);
		VkDrawIndexedError(::vulkano::command_buffer::DrawIndexedError);
	}
}
