use super::Texture;

pub struct Group {
	pub array: Vec<Texture>,
}

impl Group {
	pub fn new(cap: usize) -> Self {
		Self {
			array: Vec::with_capacity(cap),
		}
	}

	pub fn len(&self) -> usize {
		self.array.len()
	}

	pub fn capacity(&self) -> usize {
		self.array.capacity()
	}

	pub fn push(&mut self, v: Texture) {
		self.array.push(v);
	}

	pub fn position(&self, v: &Texture) -> Option<usize> {
		self.array.iter().position(|q| q == v)
	}

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
