use std::cell::Cell;

pub trait CheckSet<T> {
	fn check_set(&self, v: T) -> bool;
}

impl<T: Copy> CheckSet<T> for Cell<T> where T: PartialOrd {
	fn check_set(&self, value: T) -> bool {
		if value != self.get() {
			self.set(value);
			true
		} else {
			false
		}
	}
}