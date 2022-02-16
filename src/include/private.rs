use std::{ops::Deref, ptr::NonNull};

pub struct Dereferenceable<T: ?Sized>(NonNull<T>);
impl<T: ?Sized> Dereferenceable<T> {
	/// # Safety
	///
	/// `target` must be dereferenceable while this instance exists.
	pub unsafe fn new(target: NonNull<T>) -> Self {
		Self(target)
	}
}
impl<T: ?Sized> Deref for Dereferenceable<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { self.0.as_ref() }
	}
}
///SAFETY: This type acts as shared reference.
unsafe impl<T: ?Sized> Send for Dereferenceable<T> where T: Sync {}
///SAFETY: This type acts as shared reference.
unsafe impl<T: ?Sized> Sync for Dereferenceable<T> where T: Sync {}
