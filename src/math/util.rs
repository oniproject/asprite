use std::mem::{size_of, align_of};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub trait SliceExt {
    /// Casts an `&[T]` into an `&[U]`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following safety properties:
    ///
    ///   * The slice `self` contains valid elements of type `U`.
    ///   * The size of `T` and `U` are identical.
    ///   * The alignment of `T` is an integer multiple of the alignment of `U`.
    ///
    /// # Panics
    ///
    /// Panics if the size of `T` and `U` differ or if the alignment of `T` is
    /// not an integer multiple of `U`.
    unsafe fn cast<'a, U>(&'a self) -> &'a [U];

    /// Casts an `&mut [T]` into an `&mut [U]`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following safety properties:
    ///
    ///   * The slice `self` contains valid elements of type `U`.
    ///   * The size of `T` and `U` are identical.
    ///   * The alignment of `T` is an integer multiple of the alignment of `U`.
    ///
    /// # Panics
    ///
    /// Panics if the size of `T` and `U` differ or if the alignment of `T` is
    /// not an integer multiple of `U`.
    unsafe fn cast_mut<'a, U>(&'a mut self) -> &'a mut [U];
}

fn calc_new_len_cap<T, U>(vec: &Vec<T>) -> (usize, usize) {
    if size_of::<T>() > size_of::<U>() {
        assert!(size_of::<T>() % size_of::<U>() == 0);
        let factor = size_of::<T>() / size_of::<U>();
        (vec.len() * factor, vec.capacity() * factor)
    } else if size_of::<U>() > size_of::<T>() {
        assert!(size_of::<U>() % size_of::<T>() == 0);
        let factor = size_of::<U>() / size_of::<T>();
        (vec.len() / factor, vec.capacity() / factor)
    } else {
        (vec.len(), vec.capacity())
    }
}

fn calc_new_len<T, U>(slice: &[T]) -> usize {
    if size_of::<T>() > size_of::<U>() {
        assert!(size_of::<T>() % size_of::<U>() == 0);
        let factor = size_of::<T>() / size_of::<U>();
        slice.len() * factor
    } else if size_of::<U>() > size_of::<T>() {
        assert!(size_of::<U>() % size_of::<T>() == 0);
        let factor = size_of::<U>() / size_of::<T>();
        slice.len() / factor
    } else {
        slice.len()
    }
}

impl<T> SliceExt for [T] {
    unsafe fn cast<'a, U>(&'a self) -> &'a [U] {
        assert!(align_of::<T>() % align_of::<U>() == 0);

        let new_len = calc_new_len::<T, U>(self);
        let new_ptr = self.as_ptr() as *const U;
        from_raw_parts(new_ptr, new_len)
    }

    unsafe fn cast_mut<'a, U>(&'a mut self) -> &'a mut [U] {
        assert!(align_of::<T>() % align_of::<U>() == 0);

        let new_len = calc_new_len::<T, U>(self);
        let new_ptr = self.as_mut_ptr() as *mut U;
        from_raw_parts_mut(new_ptr, new_len)
    }
}
