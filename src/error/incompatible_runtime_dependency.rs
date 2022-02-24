use rhizome::sync::DynValue;
use std::{
	any::type_name,
	error::Error,
	fmt::{self, Debug, Display, Formatter},
	marker::PhantomData,
	pin::Pin,
};

/// Escalated when a resource tree entry for dependency injection is found, but the value fails to be downcast as expected.
pub struct IncompatibleRuntimeDependency<Expected: ?Sized> {
	expected: PhantomData<Expected>,
}
unsafe impl<Expected: ?Sized> Send for IncompatibleRuntimeDependency<Expected> {}
unsafe impl<Expected: ?Sized> Sync for IncompatibleRuntimeDependency<Expected> {}

impl<Expected: ?Sized> IncompatibleRuntimeDependency<Expected> {
	/// Creates a new instance of [`IncompatibleRuntimeDependency`] and,
	/// with the `"tracing"` feature enabled, logs this event as error.
	#[must_use]
	pub fn new_and_log(_: Pin<&DynValue>) -> Self {
		let this = Self {
			expected: PhantomData,
		};
		crate::__::tracing::error!("{}", &this);
		this
	}
}

impl<Expected: ?Sized> Error for IncompatibleRuntimeDependency<Expected> {}
impl<Expected: ?Sized> Debug for IncompatibleRuntimeDependency<Expected> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(type_name::<Self>()).finish_non_exhaustive()
	}
}
impl<Expected: ?Sized> Display for IncompatibleRuntimeDependency<Expected> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		#![allow(clippy::write_literal)] //FIXME: <https://doc.rust-lang.org/stable/core/any/fn.type_name_of_val.html> is unstable.
		write!(
			f,
			"Found incompatible runtime dependency for `{}`. The value has type id `{}`.",
			type_name::<Expected>(),
			"<type_name_of_val is unstable>",
		)
	}
}
