use super::Texture;

pub struct Group {
	pub array: Vec<Texture>,
}

impl Group {
	#[inline(always)]
	pub fn new(cap: usize) -> Self {
		Self {
			array: Vec::with_capacity(cap),
		}
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		self.array.len()
	}

	#[inline(always)]
	pub fn capacity(&self) -> usize {
		self.array.capacity()
	}

	#[inline(always)]
	pub fn push(&mut self, v: Texture) {
		self.array.push(v);
	}

	#[inline(always)]
	pub fn position(&self, v: &Texture) -> Option<usize> {
		self.array.iter().position(|q| q == v)
	}

	#[inline(always)]
	pub fn insert(&mut self, v: Texture) -> Result<usize, Texture> {
		let pos = self.position(&v);
		if self.len() != self.capacity() && pos.is_none() {
			self.array.push(v);
			Ok(self.array.len() - 1)
		} else {
			match pos {
				Some(pos) => Ok(pos),
				None => Err(v),
			}
		}
	}
}
