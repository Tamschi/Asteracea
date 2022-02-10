use rhizome::sync::DynValue;
use std::{
	any::type_name,
	error::Error,
	fmt::{self, Display},
	fmt::{Debug, Formatter},
	marker::PhantomData,
	pin::Pin,
};

pub struct IncompatibleRuntimeDependency<Expected: ?Sized> {
	expected: PhantomData<Expected>,
}
unsafe impl<Expected: ?Sized> Send for IncompatibleRuntimeDependency<Expected> {}
unsafe impl<Expected: ?Sized> Sync for IncompatibleRuntimeDependency<Expected> {}

impl<Expected: ?Sized> IncompatibleRuntimeDependency<Expected> {
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
		write!(
			f,
			"Found incompatible runtime dependency for `{}`. The value has type id `{}`.",
			type_name::<Expected>(),
			"<type_name_of_val is unstable>", // <https://doc.rust-lang.org/stable/core/any/fn.type_name_of_val.html> is unstable.
		)
	}
}
