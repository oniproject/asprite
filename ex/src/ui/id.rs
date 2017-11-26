use std::cell::Cell;
use std::usize::MAX;

// TODO: выделять половину оставшегося пространства

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(usize);

impl From<usize> for Id {
	#[inline]
	fn from(v: usize) -> Self {
		Id(v)
	}
}

pub struct Generator {
	next: Cell<usize>,
	max: usize,
}

impl Generator {
	#[inline]
	pub const fn new() -> Self {
		Self {
			next: Cell::new(0),
			max: MAX,
		}
	}

	#[inline]
	pub fn next(&self) -> Option<Id> {
		let id = self.next.get();
		if id < self.max {
			Some(self.next.replace(id + 1).into())
		} else {
			None
		}
	}

	#[inline]
	pub fn available(&self) -> Option<usize> {
		self.max.checked_sub(self.next.get())
			.and_then(|v| if v != 0 { Some(v) } else { None })
	}

	#[inline]
	fn available_count(&self, count: usize) -> Option<usize> {
		self.max.checked_sub(self.next.get() + count - 1)
			.map(|other| other.min(count))
	}

	#[inline]
	pub fn range(&self, count: usize) -> Option<Self> {
		self.available_count(count).map(|count| {
			let next = self.next.get();
			let max = next + count;
			self.next.set(max);
			Self {
				next: Cell::new(next),
				max,
			}
		})
	}
}

#[test]
fn id_gen() {
	let root = Generator::new();

	assert_eq!(root.available(), Some(::std::usize::MAX));

	assert_eq!(root.next(), Some(0.into()));
	assert_eq!(root.next(), Some(1.into()));

	{
		let x = root.range(3).unwrap();

		assert_eq!(x.available(), Some(3));
		assert_eq!(x.next(), Some(2.into()));

		assert_eq!(x.available(), Some(2));
		assert_eq!(x.next(), Some(3.into()));

		assert_eq!(x.available(), Some(1));
		assert_eq!(x.next(), Some(4.into()));

		assert_eq!(x.available(), None);
		assert_eq!(x.next(), None);

		assert!(x.range(3).is_none());

		assert_eq!(root.next(), Some(5.into()));
		assert_eq!(root.next(), Some(6.into()));
	}

	let x = root.range(3).unwrap();
	let y = x.range(2).unwrap();
	assert_eq!(y.next(), Some(7.into()));
	assert_eq!(y.next(), Some(8.into()));
	assert_eq!(y.next(), None);
	assert_eq!(x.next(), Some(9.into()));
	assert_eq!(x.next(), None);
	assert_eq!(root.next(), Some(10.into()));
}
