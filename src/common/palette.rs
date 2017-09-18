use std::ops::Index;
use std::ops::IndexMut;

pub struct Palette<T> {
	pub map: [T; 256],
	pub transparent: Option<u8>,
}

impl<T: Copy> Palette<T> {
	pub fn new<C: Into<Option<u8>>>(def: T, c: C) -> Self {
		Self {
			map: [def; 256],
			transparent: c.into(),
		}
	}
}

impl<T: Default + Copy> Palette<T> {
	pub fn empty<C: Into<Option<u8>>>(c: C) -> Self {
		Self {
			map: [Default::default(); 256],
			transparent: c.into(),
		}
	}
}

impl<T> Index<u8> for Palette<T> {
	type Output = T;
	fn index(&self, idx: u8) -> &Self::Output {
		&self.map[idx as usize]
	}
}
impl<T> IndexMut<u8> for Palette<T> {
	fn index_mut<'a>(&'a mut self, idx: u8) -> &'a mut T {
		&mut self.map[idx as usize]
	}
}

pub struct XOR([u8; 256]);

impl Index<u8> for XOR {
	type Output = u8;
	fn index(&self, idx: u8) -> &Self::Output {
		&self.0[idx as usize]
	}
}

impl XOR {
	pub fn new() -> XOR { XOR([0u8; 256]) }

	#[allow(overflowing_literals)]
	pub fn compute<F>(&mut self, diff: F) where F: Fn(u8, u8) -> isize {
		for i in 0..256 {
			self.0[i] = i as u8 ^1;
		}
		loop {
			// Find the smallest difference in the table
			// Try to pair these two colors better
			let mut found = false;
			for idx in 0..256 {
				let mut improvement = 0;
				let mut betterpair = idx;
				for i in 0..256 {
					// diffs before the swap
					let before = diff(idx, self[idx]) + diff(i, self[i]);
					// diffs after the swap
					let after = diff(idx, self[i]) + diff(i, self[idx]);

					if after - before > improvement {
						improvement = after - before;
						betterpair = i;
					}
				}

				if improvement > 0 {
					// Swapping these colors get us something "more different". Do it !
					let idx2 = self[betterpair];
					let i2 = self[idx];

					self.0[betterpair as usize] = i2;
					self.0[i2 as usize] = betterpair as u8;
					self.0[idx as usize] = idx2 as u8;
					self.0[idx2 as usize] = idx as u8;

					found = true;
				}
			}
			if !found {
				break;
			}
		}
	}
}
