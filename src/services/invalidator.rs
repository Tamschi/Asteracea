use rhizome::sync::derive_dependency;

use crate::services::ServiceHandle;

/// Call [`.invalidate()`](`Invalidator::invalidate`) to request a re-render of the injected site.
pub trait Invalidator {
	/// Requests a re-render of the injected site.
	///
	/// > The re-render *should* happen, generally sooner rather than later, but it is not entirely guaranteed.
	fn invalidate(&self);
}
derive_dependency!(dyn Invalidator);

impl<F: Fn()> Invalidator for F {
	fn invalidate(&self) {
		self()
	}
}

const _: () = {
	fn assert_usability(
		handle: &ServiceHandle<dyn Invalidator>,
	) -> &(dyn 'static + Send + Sync + std::any::Any) {
		handle
	}
};
