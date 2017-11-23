#![allow(dead_code)]
#![allow(unused_variables)]

//use super::*;

use math::*;

use std::sync::Arc;

use vulkano::device::Queue;
use vulkano::device::Device;
use vulkano::device::DeviceOwned;
use vulkano::sync::GpuFuture;
use vulkano::sync::AccessCheckError;
use vulkano::sync::AccessFlagBits;
use vulkano::sync::PipelineStages;
use vulkano::image::ImageLayout;
use vulkano::image::ImageAccess;
use vulkano::buffer::BufferAccess;
use vulkano::command_buffer::sys::UnsafeCommandBuffer;
use vulkano::command_buffer::CommandBufferExecError;
use vulkano::command_buffer::CommandBuffer;
use vulkano::command_buffer::synced::SyncCommandBuffer;
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;

type AccessResult = Result<Option<(PipelineStages, AccessFlagBits)>, AccessCheckError>;

pub struct XCommandBuffer<P = StandardCommandPoolAlloc> {
	inner: SyncCommandBuffer<P>,
}

unsafe impl<P> CommandBuffer for XCommandBuffer<P> {
	type PoolAlloc = P;

	#[inline]
	fn inner(&self) -> &UnsafeCommandBuffer<P> {
		self.inner.as_ref()
	}

	#[inline]
	fn lock_submit(&self, future: &GpuFuture, queue: &Queue) -> Result<(), CommandBufferExecError> {
		self.inner.lock_submit(future, queue)
	}

	#[inline]
	unsafe fn unlock(&self) {
		self.inner.unlock();
	}

	#[inline]
	fn check_buffer_access(&self, buffer: &BufferAccess, exclusive: bool, queue: &Queue) -> AccessResult {
		self.inner.check_buffer_access(buffer, exclusive, queue)
	}

	#[inline]
	fn check_image_access(&self, image: &ImageAccess, layout: ImageLayout, exclusive: bool, queue: &Queue) -> AccessResult {
		self.inner.check_image_access(image, layout, exclusive, queue)
	}
}

unsafe impl<P> DeviceOwned for XCommandBuffer<P> {
	#[inline]
	fn device(&self) -> &Arc<Device> {
		self.inner.device()
	}
}















use rusttype::PositionedGlyph;
use texture::Texture;

#[derive(Clone)]
pub enum Command<'glyphs, 'font>
	where 'font: 'glyphs
{
	Text {
		glyphs: &'glyphs [PositionedGlyph<'font>],
		color: [u8; 4],
	},
	Fill {
		min: Vector2<f32>,
		max: Vector2<f32>,
		color: [u8; 4],
	},
	Quad {
		texture: Texture,
		color: [u8; 4],
		pos: [Vector2<f32>; 4],
		uv: [[u16;2]; 4],
	},
}

pub struct CommandRenderer<'glyphs, 'font>
	where 'font: 'glyphs
{
	empty: Texture,
	cmd_buffer: Vec<Command<'glyphs, 'font>>
}

/*
impl CommandRenderer {
	fn _run_command<'glyphs, 'font>(&mut self, cmd: Command<'glyphs, 'font>) -> Result<()> {
		match cmd {
			Command::Text { glyphs, color } => self.glyphs(glyphs, color),
			Command::Quad { texture, color, pos, uv } => self.quad(texture, color, pos, uv),
			Command::Fill { min, max, color } => {
				let pos = [
					Vector2::new(min.x, min.y),
					Vector2::new(max.x, min.y),
					Vector2::new(max.x, max.y),
					Vector2::new(min.x, max.y),
				];
				let texture = self.empty.clone();
				self.quad(texture, color, pos, zero_uv())
			}
		}
	}

	fn glyphs<'glyphs, 'font>(&mut self, glyphs: &'glyphs [PositionedGlyph<'font>], color: [u8; 4]) -> Result<()> {
		Ok(())
	}

	pub fn quad(&mut self,
		texture: Texture,
		color: [u8; 4],
		pos: [Vector2<f32>; 4],
		uv: [[u16;2]; 4]) -> Result<()>
	{
		Ok(())
	}
}

/*
pub fn draw_indexed<V, Gp, S, Pc, Ib, I>(mut self, pipeline: Gp, dynamic: DynamicState,
											vertices: V, index_buffer: Ib, sets: S, constants: Pc)
											-> Result<Self, DrawIndexedError>
	where Gp: GraphicsPipelineAbstract + VertexSource<V> + Send + Sync + 'static + Clone, // TODO: meh for Clone
			S: DescriptorSetsCollection,
			Ib: BufferAccess + TypedBufferAccess<Content = [I]> + Send + Sync + 'static,
			I: Index + 'static
{
	unsafe {
		// TODO: must check that pipeline is compatible with render pass

		self.ensure_inside_render_pass_inline(&pipeline)?;
		let ib_infos = check_index_buffer(self.device(), &index_buffer)?;
		check_dynamic_state_validity(&pipeline, &dynamic)?;
		check_push_constants_validity(&pipeline, &constants)?;
		check_descriptor_sets_validity(&pipeline, &sets)?;
		let vb_infos = check_vertex_buffers(&pipeline, vertices)?;

		if let StateCacherOutcome::NeedChange =
			self.state_cacher.bind_graphics_pipeline(&pipeline)
		{
			self.inner.bind_pipeline_graphics(pipeline.clone());
		}

		if let StateCacherOutcome::NeedChange =
			self.state_cacher.bind_index_buffer(&index_buffer, I::ty())
		{
			self.inner.bind_index_buffer(index_buffer, I::ty())?;
		}

		let dynamic = self.state_cacher.dynamic_state(dynamic);

		push_constants(&mut self.inner, pipeline.clone(), constants);
		set_state(&mut self.inner, dynamic);
		descriptor_sets(&mut self.inner,
						&mut self.state_cacher,
						true,
						pipeline.clone(),
						sets)?;
		vertex_buffers(&mut self.inner,
						&mut self.state_cacher,
						vb_infos.vertex_buffers)?;
		// TODO: how to handle an index out of range of the vertex buffers?

		debug_assert!(self.graphics_allowed);

		self.inner
			.draw_indexed(ib_infos.num_indices as u32, 1, 0, 0, 0);
		Ok(self)
	}
}

// Shortcut function to set the push constants.
unsafe fn push_constants<P, Pl, Pc>(destination: &mut SyncCommandBufferBuilder<P>, pipeline: Pl, push_constants: Pc)
	where Pl: PipelineLayoutAbstract + Send + Sync + Clone + 'static
{
	for num_range in 0 .. pipeline.num_push_constants_ranges() {
		let range = match pipeline.push_constants_range(num_range) {
			Some(r) => r,
			None => continue,
		};

		debug_assert_eq!(range.offset % 4, 0);
		debug_assert_eq!(range.size % 4, 0);

		let data = slice::from_raw_parts(
			(&push_constants as *const Pc as *const u8).offset(range.offset as isize),
			range.size as usize);

		destination.push_constants::<_, [u8]>(pipeline.clone(),
											range.stages,
											range.offset as u32,
											range.size as u32,
											data);
	}
}

// Shortcut function to change the state of the pipeline.
unsafe fn set_state<P>(destination: &mut SyncCommandBufferBuilder<P>, dynamic: DynamicState) {
	if let Some(line_width) = dynamic.line_width {
		destination.set_line_width(line_width);
	}

	if let Some(ref viewports) = dynamic.viewports {
		destination.set_viewport(0, viewports.iter().cloned().collect::<Vec<_>>().into_iter()); // TODO: don't collect
	}

	if let Some(ref scissors) = dynamic.scissors {
		destination.set_scissor(0, scissors.iter().cloned().collect::<Vec<_>>().into_iter()); // TODO: don't collect
	}
}

unsafe fn descriptor_sets<P, Pl, S>(destination: &mut SyncCommandBufferBuilder<P>,
									state_cacher: &mut StateCacher, gfx: bool, pipeline: Pl,
									sets: S)
									-> Result<(), SyncCommandBufferBuilderError>
	where Pl: PipelineLayoutAbstract + Send + Sync + Clone + 'static,
		S: DescriptorSetsCollection
{
	let sets = sets.into_vec();

	let first_binding = {
		let mut compare = state_cacher.bind_descriptor_sets(gfx);
		for set in sets.iter() {
			compare.add(set);
		}
		compare.compare()
	};

	let first_binding = match first_binding {
		None => return Ok(()),
		Some(fb) => fb,
	};

	let mut sets_binder = destination.bind_descriptor_sets();
	for set in sets.into_iter().skip(first_binding as usize) {
		sets_binder.add(set);
	}
	sets_binder
		.submit(gfx, pipeline.clone(), first_binding, iter::empty())?;
	Ok(())
}
*/

*/
