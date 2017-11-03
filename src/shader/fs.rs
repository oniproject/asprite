use std::sync::Arc;
//use std::vec::IntoIter as VecIntoIter;
//use std::hash::Hash;
use std::borrow::Cow;
//use vulkano::device::Device;
use vulkano::descriptor::descriptor::DescriptorDesc;
use vulkano::descriptor::descriptor::DescriptorDescTy;
//use vulkano::descriptor::descriptor::DescriptorBufferDesc;
use vulkano::descriptor::descriptor::DescriptorImageDesc;
use vulkano::descriptor::descriptor::DescriptorImageDescDimensions;
use vulkano::descriptor::descriptor::DescriptorImageDescArray;
use vulkano::descriptor::descriptor::ShaderStages;
//use vulkano::descriptor::descriptor_set::DescriptorSet;
//use vulkano::descriptor::descriptor_set::UnsafeDescriptorSet;
//use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
//use vulkano::descriptor::pipeline_layout::PipelineLayout;
use vulkano::descriptor::pipeline_layout::PipelineLayoutDesc;
use vulkano::descriptor::pipeline_layout::PipelineLayoutDescPcRange;
use vulkano::pipeline::shader::SpecializationConstants as SpecConstsTrait;
use vulkano::pipeline::shader::SpecializationMapEntry;
use vulkano::pipeline::shader::ShaderModule;

use vulkano::format::Format;

pub const TEXTURE_COUNT: usize = 16;

def!(MainInput MainInputIter
	tex_coords => Format::R32G32Sfloat,
	tex_color => Format::R32G32B32A32Sfloat,
	tex_id => Format::R32Uint,
);

def!(MainOutput MainOutputIter
	f_color => ::vulkano::format::Format::R32G32B32A32Sfloat,
);

pub struct Shader {
	pub shader: Arc<ShaderModule>,
}
impl Shader {
	/// Loads the shader in Vulkan as a `ShaderModule`.
	#[inline]
	pub fn load(device: Arc<::vulkano::device::Device>)
				-> Result<Shader, ::vulkano::OomError> {
		unsafe {
			let data = include_bytes!("./spritebatch.frag.spv");
			Ok(Shader {
				shader: match ShaderModule::new(device, &data[..]) {
					Ok(val) => val,
					Err(err) => {
						return Err(From::from(err))
					}
				},
			})
		}
	}
	/// Returns a logical struct describing the entry point named `main`.
	#[inline]
	pub fn main_entry_point
		(&self)
		-> ::vulkano::pipeline::shader::GraphicsEntryPoint<(), MainInput, MainOutput, Layout> {
		unsafe {
			static NAME: [u8; 5] = *b"main\0";
			self.shader
				.graphics_entry_point(::std::ffi::CStr::from_ptr(NAME.as_ptr() as *const _),
									MainInput,
									MainOutput,
									Layout(ShaderStages {
												fragment: true,
												..ShaderStages::none()
											}),
									::vulkano::pipeline::shader::GraphicsShaderType::Fragment)
		}
	}
}

pub mod ty {}

#[derive(Clone, Debug)]
pub struct Layout(pub ShaderStages);
unsafe impl PipelineLayoutDesc for Layout {
	fn num_sets(&self) -> usize {
		2
	}
	fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
		match set {
			0 => Some(0),
			1 => Some(1),
			_ => None,
		}
	}
	fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
		match (set, binding) {
			(1, 0) => Some(DescriptorDesc {
				ty: DescriptorDescTy::CombinedImageSampler(DescriptorImageDesc{
					sampled: true,
					dimensions: DescriptorImageDescDimensions::TwoDimensional,
					format: None,
					multisampled: false,
					array_layers: DescriptorImageDescArray::NonArrayed,
				}),
				array_count: TEXTURE_COUNT as u32,
				stages: self.0.clone(),
				readonly: true,
			}),
		_ => None,
	}
}
fn num_push_constants_ranges(&self) -> usize {
		0
	}
	fn push_constants_range(&self, num: usize) -> Option<PipelineLayoutDescPcRange> {
		if num != 0 || 0 == 0 {
			return None;
		}
		Some(PipelineLayoutDescPcRange {
				offset: 0,
				size: 0,
				stages: ShaderStages::all(),
			})
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SpecializationConstants {}
impl Default for SpecializationConstants {
	fn default() -> SpecializationConstants {
		SpecializationConstants {}
	}
}
unsafe impl SpecConstsTrait for SpecializationConstants {
	fn descriptors() -> &'static [SpecializationMapEntry] {
		static DESCRIPTORS: [SpecializationMapEntry; 0] = [];
		&DESCRIPTORS
	}
}
