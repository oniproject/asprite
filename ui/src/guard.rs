use std::mem::ManuallyDrop;
use std::mem::{uninitialized, replace, drop};

macro_rules! deref {
	($e:expr) => {
		let _guard = Guard::new(|| $e);
	}
}

#[must_use]
pub struct Guard<F: FnOnce()> {
	f: ManuallyDrop<F>,
}

impl<F: FnOnce()> Drop for Guard<F> {
	fn drop(&mut self) {
		let f = replace(&mut self.f, unsafe { uninitialized() });
		let f = ManuallyDrop::into_inner(f);
		f();
	}
}

impl<F: FnOnce()> Guard<F> {
	pub fn new(f: F) -> Self {
		Self { f: ManuallyDrop::new(f)  }
	}
	pub fn run(self) {
		drop(self);
	}
	pub fn join<B: FnOnce()>(self, b: Guard<B>) -> Guard<impl FnOnce()> {
		join_guard(self, b)
	}
}

pub fn join_guard<A: FnOnce(), B: FnOnce()>(a: Guard<A>, b: Guard<B>) -> Guard<impl FnOnce()> {
	Guard::new(|| {
		a.run();
		b.run();
	})
}
