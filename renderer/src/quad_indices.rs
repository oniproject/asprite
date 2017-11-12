use super::*;

#[derive(Clone)]
pub struct QuadIBO<T>(Index<T>);

impl QuadIBO<u16> {
	pub fn new(queue: Arc<Queue>, cap: usize) -> Result<(Self, BoxFuture)> {
		let (index, future) = ImmutableBuffer::from_iter(
			QuadIndices(0u16, cap),
			BufferUsage::index_buffer(),
			queue,
		)?;
		Ok((QuadIBO(index), Box::new(future)))
	}
	pub fn slice(&self, count: usize) -> Option<ChunkIBO<u16>> {
		self.0.clone().into_buffer_slice().slice(0..count)
	}
}

struct QuadIndices<T>(pub T, pub usize);

impl ExactSizeIterator for QuadIndices<u16> {}
impl ExactSizeIterator for QuadIndices<u32> {}

impl Iterator for QuadIndices<u16> {
	type Item = u16;
	fn next(&mut self) -> Option<Self::Item> {
		let i = self.0/6 * 4 + match self.0%6 {
			0 => 0,
			1 => 1,
			2 => 2,
			3 => 0,
			4 => 2,
			5 => 3,
			_ => unreachable!(),
		};
		self.0 += 1;
		if self.0 <= self.1 as u16 {
			Some(i)
		} else {
			None
		}
	}
	fn size_hint(&self) -> (usize, Option<usize>) { (self.1, Some(self.1)) }
}

impl Iterator for QuadIndices<u32> {
	type Item = u32;
	fn next(&mut self) -> Option<Self::Item> {
		let i = self.0/6 * 4 + match self.0%6 {
			0 => 0,
			1 => 1,
			2 => 2,
			3 => 0,
			4 => 2,
			5 => 3,
			_ => unreachable!(),
		};
		self.0 += 1;
		if self.0 <= self.1 as u32 {
			Some(i)
		} else {
			None
		}
	}
	fn size_hint(&self) -> (usize, Option<usize>) { (self.1, Some(self.1)) }
}

#[test]
fn gen() {
	fn create_indices_for_quads(indices: &mut [u16], size: usize) {
		// the total number of indices in our array, there are 6 points per quad.
		let total = size * 6;
		assert!(total == indices.len());

		// fill the indices with the quads to draw
		let mut i = 0;
		let mut j = 0;
		while i < total {
			indices[i + 0] = j + 0;
			indices[i + 1] = j + 1;
			indices[i + 2] = j + 2;
			indices[i + 3] = j + 0;
			indices[i + 4] = j + 2;
			indices[i + 5] = j + 3;
			i += 6;
			j += 4;
		}
	}

	let size = 4;
	let mut idx1 = vec![0; size*6];
	create_indices_for_quads(&mut idx1[..], size);
	for i in 0..size {
		println!("{:?}", &idx1[i*6..(i+1)*6]);
	}

	println!("---");

	let idx2: Vec<_> = QuadIndices(0u16, size*6).collect();
	for i in 0..size {
		println!("{:?}", &idx2[i*6..(i+1)*6]);
	}

	assert_eq!(idx1, idx2);
}
