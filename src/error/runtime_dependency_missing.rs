use std::{
	any::type_name,
	error::Error,
	fmt::{self, Debug, Display, Formatter},
	marker::PhantomData,
};

pub struct RuntimeDependencyMissing<Expected: ?Sized> {
	expected: PhantomData<Expected>,
}
unsafe impl<Expected: ?Sized> Send for RuntimeDependencyMissing<Expected> {}
unsafe impl<Expected: ?Sized> Sync for RuntimeDependencyMissing<Expected> {}

impl<Expected: ?Sized> RuntimeDependencyMissing<Expected> {
	pub fn new_and_log() -> Self {
		let this = Self {
			expected: PhantomData,
		};
		crate::__::tracing::error!("{}", &this);
		this
	}
}

impl<Expected: ?Sized> Error for RuntimeDependencyMissing<Expected> {}
impl<Expected: ?Sized> Debug for RuntimeDependencyMissing<Expected> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(type_name::<Self>()).finish_non_exhaustive()
	}
}
impl<Expected: ?Sized> Display for RuntimeDependencyMissing<Expected> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"Failed to find runtime dependency of type `{}`",
			type_name::<Expected>()
		)
	}
}
