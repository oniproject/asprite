use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_pool::CpuBufferPoolChunk;
use vulkano::memory::MemoryPool;
use vulkano::memory::pool::StdMemoryPool;
use vulkano::device::Device;

use std::sync::Arc;

use errors::*;

pub struct VBO<T, A = Arc<StdMemoryPool>>
	where A: MemoryPool
{
	pub vertices: Vec<T>,
	pub vertex: CpuBufferPool<T, A>,
}

impl<T> VBO<T> {
	#[inline]
	pub fn new(device: Arc<Device>, capacity: usize) -> Self {
		let vertex = CpuBufferPool::vertex_buffer(device);
		let vertices = Vec::with_capacity(capacity);
		Self { vertex, vertices }
	}

	#[inline]
	pub fn with_usage(device: Arc<Device>, usage: BufferUsage, capacity: usize) -> Self {
		let vertex = CpuBufferPool::new(device, usage);
		let vertices = Vec::with_capacity(capacity);
		Self { vertex, vertices }
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.vertices.is_empty()
	}

	#[inline]
	pub fn is_full(&self) -> bool {
		self.vertices.len() == self.vertices.capacity()
	}

	#[inline]
	pub fn push(&mut self, v: T) {
		self.vertices.push(v)
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.vertices.len()
	}
}

impl<T, A: MemoryPool> VBO<T, A> {
	#[inline]
	pub fn flush(&mut self) -> Result<CpuBufferPoolChunk<T, A>> {
		Ok(self.vertex.chunk(self.vertices.drain(..))?)
	}
}
