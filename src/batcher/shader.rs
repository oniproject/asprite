use super::*;

static MAIN: [u8; 5] = *b"main\0";

macro_rules! def {
	(@step $_idx:expr, $self:expr, ) => {};
	(@step $idx:expr, $self:expr, $name:ident => $format:path, $($_name:ident => $_format:path,)*) => {
		if $self.num == $idx {
			$self.num += 1;
			return Some($crate::vulkano::pipeline::shader::ShaderInterfaceDefEntry {
				location: $idx..$idx+1,
				format: $format,
				name: Some(Cow::Borrowed(stringify!($name))),
			});
		}
		def!(@step $idx + 1, $self, $($_name => $_format,)*)
	};

	// counting
	(@step $idx:expr, ) => { $idx };
	(@step $idx:expr, $name:ident => $format:path, $($_name:ident => $_format:path,)*) => {
		def!(@step $idx + 1, $($_name => $_format,)*)
	};

	($class:ident $iter:ident $( $name:ident => $format:path, )*) => {

		#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
		pub struct $class;
		unsafe impl $crate::vulkano::pipeline::shader::ShaderInterfaceDef for $class {
			type Iter = $iter;
			fn elements(&self) -> $iter {
				$iter { num: 0 }
			}
		}

		#[derive(Debug, Copy, Clone)]
		pub struct $iter {
			num: u16,
		}
		impl Iterator for $iter {
			type Item = $crate::vulkano::pipeline::shader::ShaderInterfaceDefEntry;
			#[inline]
			fn next(&mut self) -> Option<Self::Item> {
				def!(@step 0, self, $($name => $format,)*);
				None
			}
			#[inline]
			fn size_hint(&self) -> (usize, Option<usize>) {
				let len = (
					def!(@step 0, $($name => $format,)*)
					- self.num) as usize;
				(len, Some(len))
			}
		}
		impl ExactSizeIterator for $iter {}
	};
}

def!(Vert2Frag Vert2FragIter
	tex_coords => Format::R32G32Sfloat,
	tex_color => Format::R32G32B32A32Sfloat,
	tex_id => Format::R32Uint,
);

def!(FragOutput FragOutputIter
	f_color => Format::R32G32B32A32Sfloat,
);

def!(VertInput VertInputIter
	position => Format::R32G32Sfloat,
	uv => Format::R16G16Unorm,
	color => Format::R8G8B8A8Unorm,
	texture => Format::R32Uint,
);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Uniform {
	pub proj: [[f32; 4]; 4],
}

pub struct Shader {
	pub frag: Arc<ShaderModule>,
	pub vert: Arc<ShaderModule>,
}

impl Shader {
	/// Loads the shader in Vulkan as a `ShaderModule`.
	#[inline]
	pub fn load(device: Arc<Device>) -> Result<Shader> {
		unsafe {
			let frag = include_bytes!("./spritebatch.frag.spv");
			let vert = include_bytes!("./spritebatch.vert.spv");

			let frag = ShaderModule::new(device.clone(), &frag[..])?;
			let vert = ShaderModule::new(device, &vert[..])?;

			Ok(Self { frag, vert })
		}
	}

	/// Returns a logical struct describing the entry point named `main`.
	#[inline]
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
	#[inline]
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
unsafe impl PipelineLayoutDesc for FragmentLayout {
	#[inline]
	fn num_sets(&self) -> usize {
		2
	}
	#[inline]
	fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
		match set {
			0 => Some(0),
			1 => Some(1),
			_ => None,
		}
	}
	#[inline]
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
				array_count: self.count,
				stages: self.stages.clone(),
				readonly: true,
			}),
			_ => None,
		}
	}
	#[inline]
	fn num_push_constants_ranges(&self) -> usize {
		0
	}
	#[inline]
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
	#[inline]
	fn num_sets(&self) -> usize {
		1
	}
	#[inline]
	fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
		match set {
			0 => Some(1),
			_ => None,
		}
	}
	#[inline]
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
	#[inline]
	fn num_push_constants_ranges(&self) -> usize {
		0
	}
	#[inline]
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
	#[inline]
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
