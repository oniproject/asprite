use super::Texture;
use smallvec::SmallVec;
use std::sync::Arc;

pub struct Group {
	pub array: SmallVec<[Texture; 16]>,
	pub cap: usize,
}

impl Group {
	#[inline(always)]
	pub fn new(cap: usize) -> Self {
		Self {
			array: SmallVec::with_capacity(cap),
			cap,
		}
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		self.array.len()
	}

	#[inline(always)]
	pub fn capacity(&self) -> usize {
		self.cap
	}

	#[inline(always)]
	pub fn push(&mut self, v: Texture) {
		self.array.push(v);
	}

	#[inline(always)]
	pub fn position(&self, v: &Texture) -> Option<usize> {
		self.array.iter().position(|q| Arc::ptr_eq(&q.texture, &v.texture))
	}

	#[inline(always)]
	pub fn insert(&mut self, v: Texture) -> Result<usize, Texture> {
		let pos = self.position(&v);
		if self.len() != self.cap && pos.is_none() {
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
