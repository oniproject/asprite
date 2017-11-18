extern crate cgmath;
extern crate image;
extern crate winit;

#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;

use vulkano_win::VkSurfaceBuild;
use vulkano::sync::GpuFuture;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::swapchain::{
	Swapchain,
	SurfaceTransform,
	PresentMode,
	SwapchainCreationError,
	acquire_next_image,
	AcquireError,
};

use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};

use std::sync::Arc;

fn main() {
	// The start of this example is exactly the same as `triangle`. You should read the
	// `triangle` example if you haven't done so yet.

	let extensions = vulkano_win::required_extensions();
	let instance = Instance::new(None, &extensions, &[]).expect("failed to create instance");

	let physical = PhysicalDevice::enumerate(&instance)
							.next().expect("no device available");
	println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

	let mut events_loop = winit::EventsLoop::new();
	let window = winit::WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();

	let mut dimensions;

	let queue = physical.queue_families()
		.find(|&q| q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false))
		.expect("couldn't find a graphical queue family");

	let device_ext = DeviceExtensions {
		khr_swapchain: true,
		.. DeviceExtensions::none()
	};
	let (device, mut queues) = Device::new(
			physical, physical.supported_features(),
			&device_ext, [(queue, 0.5)].iter().cloned())
		.expect("failed to create device");
	let queue = queues.next().unwrap();

	let (mut sc, mut images) = {
		let caps = window.surface()
			.capabilities(physical)
			.expect("failed to get surface capabilities");

		dimensions = caps.current_extent.unwrap_or([1024, 768]);
		let usage = caps.supported_usage_flags;
		let alpha = caps.supported_composite_alpha.iter().next().unwrap();
		let format = caps.supported_formats[0].0;

		Swapchain::new(
				device.clone(), window.surface().clone(), caps.min_image_count,
				format, dimensions, 1,
				usage, &queue, SurfaceTransform::Identity,
				alpha,
				PresentMode::Fifo, true, None)
			.expect("failed to create swapchain")
	};

	#[derive(Debug, Clone)]
	struct Vertex {
		position: [f32; 2],
		color: [f32; 3],
		alpha: f32,
	}
	impl_vertex!(Vertex, position, color, alpha);

	let vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer::<[Vertex]>::from_iter(
			device.clone(),
			vulkano::buffer::BufferUsage::all(), [
					Vertex { position: [-0.5, -0.5 ], color: [1.0, 1.0, 1.0], alpha: 1.0 },
					Vertex { position: [-0.5,  0.5 ], color: [1.0, 1.0, 1.0], alpha: 1.0 },
					Vertex { position: [ 0.5, -0.5 ], color: [1.0, 1.0, 1.0], alpha: 1.0 },
					Vertex { position: [ 0.5,  0.5 ], color: [1.0, 1.0, 1.0], alpha: 1.0 },
				].iter().cloned()
			)
		.expect("failed to create buffer");

	let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
	let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

	let renderpass = Arc::new(
		single_pass_renderpass!(device.clone(),
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: sc.format(),
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		).unwrap()
	);

	let (texture, tex_future) = {
		let image = image::load_from_memory_with_format(
				include_bytes!("../../f/tileset_1bit.png"),
				image::ImageFormat::PNG)
			.unwrap()
			.to_rgba();
		let (w, h) = image.dimensions();
		let image_data = image.into_raw().clone();

		vulkano::image::ImmutableImage::from_iter(
			image_data.iter().cloned(),
			vulkano::image::Dimensions::Dim2d { width: w, height: h},
			vulkano::format::R8G8B8A8Srgb,
			queue.clone()).unwrap()
	};

	let sampler = Sampler::new(
			device.clone(),
			Filter::Nearest, Filter::Nearest,
			MipmapMode::Nearest,
			SamplerAddressMode::Repeat, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
			0.0, 1.0, 0.0, 0.0)
		.unwrap();

	let pipeline = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
		.vertex_input_single_buffer::<Vertex>()
		.vertex_shader(vs.main_entry_point(), ())
		.triangle_strip()
		.viewports_dynamic_scissors_irrelevant(1)
		.fragment_shader(fs.main_entry_point(), ())
		.blend_alpha_blending()
		.render_pass(vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
		.build(device.clone())
		.unwrap());

	let set = Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
		.add_sampled_image(texture.clone(), sampler.clone()).unwrap()
		.build().unwrap()
	);

	let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_,_>>>> = None;
	let mut recreate_swapchain = false;
	let mut previous_frame_end = Box::new(tex_future) as Box<GpuFuture>;

	loop {
		previous_frame_end.cleanup_finished();
		if recreate_swapchain {
			dimensions = window.surface().capabilities(physical)
				.expect("failed to get surface capabilities")
				.current_extent.unwrap_or([1024, 768]);

			let (new_swapchain, new_images) = match sc.recreate_with_dimension(dimensions) {
				Ok(r) => r,
				Err(SwapchainCreationError::UnsupportedDimensions) => {
					continue;
				},
				Err(err) => panic!("{:?}", err)
			};

			std::mem::replace(&mut sc, new_swapchain);
			std::mem::replace(&mut images, new_images);

			framebuffers = None;

			recreate_swapchain = false;
		}

		if framebuffers.is_none() {
			let new_framebuffers = Some(images.iter().map(|image| {
				Arc::new(vulkano::framebuffer::Framebuffer::start(renderpass.clone())
						.add(image.clone()).unwrap()
						.build().unwrap())
			}).collect::<Vec<_>>());
			std::mem::replace(&mut framebuffers, new_framebuffers);
		}

		let (image_num, future) = match acquire_next_image(sc.clone(), None) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				recreate_swapchain = true;
				continue;
			},
			Err(err) => panic!("{:?}", err)
		};

		let cb = vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
			.unwrap()
			.begin_render_pass(
				framebuffers.as_ref().unwrap()[image_num].clone(),
				false,
				vec![[0.0, 0.0, 1.0, 1.0].into()]).unwrap()
			.draw(pipeline.clone(),
				vulkano::command_buffer::DynamicState {
					line_width: None,
					viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
						origin: [0.0, 0.0],
						dimensions: [dimensions[0] as f32, dimensions[1] as f32],
						depth_range: 0.0 .. 1.0,
					}]),
					scissors: None,
				},
				vertex_buffer.clone(),
				set.clone(), ()).unwrap()
			.end_render_pass().unwrap()
			.build().unwrap();

		let future = previous_frame_end.join(future)
			.then_execute(queue.clone(), cb).unwrap()
			.then_swapchain_present(queue.clone(), sc.clone(), image_num)
			.then_signal_fence_and_flush().unwrap();
		previous_frame_end = Box::new(future) as Box<_>;

		let mut done = false;
		events_loop.poll_events(|ev| {
			match ev {
				winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } => done = true,
				_ => ()
			}
		});
		if done { return; }
	}
}

mod vs {
	#[derive(VulkanoShader)]
	#[ty = "vertex"]
	#[src = "
#version 450
layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;
void main() {
	gl_Position = vec4(position, 0.0, 1.0);
	tex_coords = position + vec2(0.5);
}
"]
	struct Dummy;
}

mod fs {
	#[derive(VulkanoShader)]
	#[ty = "fragment"]
	#[src = "
#version 450
layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding = 0) uniform sampler2D tex;
void main() {
	f_color = texture(tex, tex_coords);
}
"]
	struct Dummy;
}
