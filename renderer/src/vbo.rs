use super::*;

pub struct VBO<T, A = Arc<StdMemoryPool>>
	where A: MemoryPool
{
	vertices: Vec<T>,
	vertex: CpuBufferPool<T, A>,
}

impl<T> VBO<T> {
	pub fn new(device: Arc<Device>, capacity: usize) -> Self {
		let vertex = CpuBufferPool::vertex_buffer(device);
		let vertices = Vec::with_capacity(capacity);
		Self { vertex, vertices }
	}

	pub fn is_empty(&self) -> bool {
		self.vertices.is_empty()
	}

	pub fn is_full(&self) -> bool {
		self.vertices.len() == self.vertices.capacity()
	}

	pub fn push(&mut self, v: T) {
		self.vertices.push(v)
	}

	pub fn len(&self) -> usize {
		self.vertices.len()
	}
}

impl<T, A: MemoryPool> VBO<T, A> {
	pub fn flush(&mut self) -> Result<CpuBufferPoolChunk<T, A>> {
		Ok(self.vertex.chunk(self.vertices.drain(..))?)
	}
}
