#[macro_export]
macro_rules! smallset {
	($name:ident [ $t:ty; $size:expr ]) => {
		pub struct $name {
			pub array: Vec<$t>,
		}

		#[allow(dead_code)]
		impl $name {
			#[inline(always)]
			pub fn new() -> Self {
				Self {
					array: Vec::with_capacity($size),
				}
			}

			#[inline(always)]
			pub fn len(&self) -> usize {
				self.array.len()
			}

			#[inline(always)]
			pub fn capacity(&self) -> usize {
				$size
			}

			#[inline(always)]
			pub fn clear(&mut self) {
				self.array.clear();
			}

			#[inline(always)]
			pub fn position(&self, v: &$t) -> Option<usize> {
				self.array.iter().position(|q| q == v)
			}

			#[inline(always)]
			pub fn insert(&mut self, v: $t) -> Option<usize> {
				let pos = self.position(&v);
				if self.len() != self.capacity() && pos.is_none() {
					self.array.push(v);
					Some(self.array.len() - 1)
				} else {
					pos
				}
			}

			#[inline(always)]
			pub fn insert_r(&mut self, v: $t) -> Result<usize, $t> {
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

			#[inline(always)]
			pub fn to_slice(&self) -> &[$t] {
				&self.array[..]
			}

			#[inline(always)]
			pub fn is_empty(&self) -> bool {
				self.len() == 0
			}

			#[inline(always)]
			pub fn is_full(&self) -> bool {
				self.len() == self.capacity()
			}
		}
	};
}

#[test]
fn common() {
	smallset!(Set[usize; 3]);

	let mut v = Set{
		len: 0,
		array: [None; 3],
	};

	assert!(v.is_empty() && !v.is_full());

	assert_eq!(v.insert(5), Some(0));
	assert!(!v.is_empty() && !v.is_full());

	assert_eq!(v.insert(5), Some(0));
	assert!(!v.is_empty() && !v.is_full());

	assert_eq!(v.insert(4), Some(1));
	assert!(!v.is_empty() && !v.is_full());

	assert_eq!(v.insert(3), Some(2));
	assert!(!v.is_empty() && v.is_full());

	assert_eq!(v.insert(7), None);
	assert!(!v.is_empty() && v.is_full());

	assert_eq!(v.to_slice(), &[Some(5), Some(4), Some(3)]);
}
