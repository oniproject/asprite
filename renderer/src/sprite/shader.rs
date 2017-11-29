use super::*;
use std::sync::Arc;

use vulkano::device::Device;
use vulkano::format::Format;

use vulkano::descriptor::pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange};
use vulkano::descriptor::descriptor::{DescriptorBufferDesc, ShaderStages};
use vulkano::descriptor::descriptor::{DescriptorDesc, DescriptorDescTy};
use vulkano::descriptor::descriptor::{DescriptorImageDesc, DescriptorImageDescDimensions, DescriptorImageDescArray};

use vulkano::pipeline::shader::SpecializationConstants as SpecConstsTrait;
use vulkano::pipeline::shader::SpecializationMapEntry;
use vulkano::pipeline::shader::ShaderModule;
use vulkano::pipeline::shader::GraphicsEntryPoint;
use vulkano::pipeline::shader::GraphicsShaderType;

#[derive(Derivative, Clone, Copy)]
#[derivative(Default)]
pub struct Vertex {
	// 4*2 + 2*2 + 4*1 + 4 = 20
	// 20 * 4 = 80 bytes per sprite instead 128
	#[derivative(Default(value="[0.0; 2]"))]
	pub position: [f32; 2],
	#[derivative(Default(value="[0; 2]"))]
	pub uv: [u16; 2],
	#[derivative(Default(value="[0xFF; 4]"))]
	pub color: [u8; 4],
	#[derivative(Default(value="0"))]
	pub texture: u32,
}

impl_vertex!(Vertex, position, uv, color, texture);

def!(VertInput VertInputIter
	position => Format::R32G32Sfloat,
	uv => Format::R16G16Unorm,
	color => Format::R8G8B8A8Unorm,
	//color => Format::R8G8B8A8Srgb,
	texture => Format::R32Uint,
);

def!(Vert2Frag Vert2FragIter
	tex_coords => Format::R16G16Unorm,
	//tex_color => Format::R32G32B32A32Sfloat,
	tex_color => Format::R8G8B8A8Srgb,
	//tex_color => Format::R8G8B8A8Unorm,
	tex_id => Format::R32Uint,
);

def!(FragOutput FragOutputIter
	//f_color => Format::R32G32B32A32Sfloat,
	f_color => Format::R8G8B8A8Srgb,
	//f_color => Format::R8G8B8A8Unorm,
);

pub struct Shader {
	pub frag: Arc<ShaderModule>,
	pub vert: Arc<ShaderModule>,
}

impl Shader {
	/// Loads the shader in Vulkan as a `ShaderModule`.
	pub fn load(device: Arc<Device>) -> Result<Shader> {
		unsafe {
			let frag = include_bytes!("./shader.frag.spv");
			let vert = include_bytes!("./shader.vert.spv");

			let frag = ShaderModule::new(device.clone(), &frag[..])?;
			let vert = ShaderModule::new(device, &vert[..])?;

			Ok(Self { frag, vert })
		}
	}

	/// Returns a logical struct describing the entry point named `main`.
	pub fn vert_entry_point(&self) -> (GraphicsEntryPoint<(), VertInput, Vert2Frag, VertexLayout>, ()) {
		(
		unsafe {
			self.vert.graphics_entry_point(
				::std::ffi::CStr::from_ptr(MAIN.as_ptr() as *const _),
				VertInput,
				Vert2Frag,
				VertexLayout(ShaderStages {
							vertex: true,
							..ShaderStages::none()
						}),
				GraphicsShaderType::Vertex)
		},
		())
	}

	/// Returns a logical struct describing the entry point named `main`.
	pub fn frag_entry_point(&self, count: u32) -> (GraphicsEntryPoint<TextureCount, Vert2Frag, FragOutput, FragmentLayout>, TextureCount) {
		(unsafe {
			self.frag.graphics_entry_point(
				::std::ffi::CStr::from_ptr(MAIN.as_ptr() as *const _),
				Vert2Frag,
				FragOutput,
				FragmentLayout{
					stages: ShaderStages { fragment: true, ..ShaderStages::none() },
					count,
				},
				GraphicsShaderType::Fragment)
		},
		TextureCount { count })
	}
}

#[derive(Clone, Debug)]
pub struct FragmentLayout {
	pub stages: ShaderStages,
	pub count: u32,
}

impl FragmentLayout {
	#[inline]
	pub fn frag_desc(&self) -> DescriptorDesc {
		DescriptorDesc {
			ty: DescriptorDescTy::CombinedImageSampler(DescriptorImageDesc{
				sampled: true,
				dimensions: DescriptorImageDescDimensions::TwoDimensional,
				format: None,
				multisampled: false,
				array_layers: DescriptorImageDescArray::NonArrayed,
			}),
			array_count: self.count,
			stages: self.stages.clone(),
			readonly: true,
		}
	}
}


unsafe impl PipelineLayoutDesc for FragmentLayout {
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
			(1, 0) => Some(self.frag_desc()),
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

#[derive(Debug, Clone)]
pub struct VertexLayout(pub ShaderStages);
unsafe impl PipelineLayoutDesc for VertexLayout {
	fn num_sets(&self) -> usize {
		1
	}
	fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
		match set {
			0 => Some(1),
			_ => None,
		}
	}
	fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
		match (set, binding) {
			(0, 0) => {
				Some(DescriptorDesc {
						ty: DescriptorDescTy::Buffer(DescriptorBufferDesc {
														dynamic: Some(false),
														storage: false,
													}),
						array_count: 1,
						stages: self.0.clone(),
						readonly: true,
					})
			}
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
pub struct TextureCount {
	pub count: u32,
}
unsafe impl SpecConstsTrait for TextureCount {
	fn descriptors() -> &'static [SpecializationMapEntry] {
		static DESCRIPTORS: [SpecializationMapEntry; 1] = [
			SpecializationMapEntry {
				constant_id: 0,
				offset: 0,
				size: 4,
			},
		];
		&DESCRIPTORS
	}
}
