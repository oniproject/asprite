#[macro_export]
macro_rules! smallset {
	($name:ident <$t:ty> [ $fill:expr; $size:expr ]) => {
		pub struct $name {
			pub len: usize,
			pub array: [Option<$t>; $size],
		}

		impl $name {
			/*
			#[inline(always)]
			pub fn new() -> Self {
				Self {
					array: [$fill; $size],
					len: 0,
				}
			}
			*/

			#[inline(always)]
			pub fn len(&self) -> usize {
				self.len
			}

			#[inline(always)]
			pub fn capacity(&self) -> usize {
				$size
			}

			#[inline(always)]
			pub fn position(&self, v: &$t) -> Option<usize> {
				for i in 0..$size {
					match self.array[i] {
						Some(ref q) if q == v => return Some(i),
						None => return None,
						_ => (),
					}
				}
				None
			}

			#[inline(always)]
			pub fn insert(&mut self, v: $t) -> Option<usize> {
				let pos = self.position(&v);
				if self.len() != self.capacity() && pos.is_none() {
					self.array[self.len] = Some(v);
					self.len += 1;
					Some(self.len - 1)
				} else {
					pos
				}
			}

			#[inline(always)]
			pub fn to_slice(&self) -> &[Option<$t>] {
				&self.array[..self.len]
			}

			#[inline(always)]
			pub fn is_empty(&self) -> bool {
				self.len == 0
			}

			#[inline(always)]
			pub fn is_full(&self) -> bool {
				self.len == self.array.len()
			}
		}
	};
}

#[test]
fn common() {
	smallset!(Set<usize>[None; 3]);

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