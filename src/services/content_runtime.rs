use rhizome::sync::derive_dependency;

use crate::include::async_::ContentFuture;

/// A resource used by [`Suspense`](`crate::components::Suspense`) to schedule [`ContentFuture`]s.
pub trait ContentRuntime {
	fn start_content_future(&self, content_future: ContentFuture);
}
derive_dependency!(dyn ContentRuntime);

impl<F: Fn(ContentFuture)> ContentRuntime for F {
	fn start_content_future(&self, content_future: ContentFuture) {
		self(content_future)
	}
}
