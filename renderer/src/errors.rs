error_chain! {
	foreign_links {
		//NoneError(::std::option::NoneError);
		//CacheWriteErr(::rusttype::gpu_cache::CacheWriteErr);
		ImageError(::image::ImageError);
		VkImageCreationError(::vulkano::image::ImageCreationError);
		VkSamplerCreationError(::vulkano::sampler::SamplerCreationError);
		VkOomError(::vulkano::OomError);
		VkGraphicsPipelineCreationError(::vulkano::pipeline::GraphicsPipelineCreationError);
		VkDeviceMemoryAllocError(::vulkano::memory::DeviceMemoryAllocError);
		VkPersistentDescriptorSetError(::vulkano::descriptor::descriptor_set::PersistentDescriptorSetError);
		VkPersistentDescriptorSetBuildError(::vulkano::descriptor::descriptor_set::PersistentDescriptorSetBuildError);
		VkDrawIndexedError(::vulkano::command_buffer::DrawIndexedError);
		VkAutoCommandBufferBuilderContextError(::vulkano::command_buffer::AutoCommandBufferBuilderContextError);
		VkCopyBufferImageError(::vulkano::command_buffer::CopyBufferImageError);
		VkBeginRenderPassError(::vulkano::command_buffer::BeginRenderPassError);
		VkBuildError(::vulkano::command_buffer::BuildError);
	}
}
