use crate::include::async_::ContentFuture;
use rhizome::sync::derive_dependency;

/// A resource used by [`Suspense`](`crate::components::Suspense`) to schedule [`ContentFuture`]s.
///
/// # Implementation Guidelines
///
/// There are no specific requirements regarding the behaviour of a [`ContentRuntime`] implementation.
/// For example, it could be entirely synchronous/blocking, or it could choose to never poll any [`ContentFuture`]s at all.
///
/// However, when running in a browser, I suggest polling the [`ContentFuture`] once synchronously at the beginning!
/// Doing so avoids a flash of non-loaded-content in for example [`Suspense`][`crate::components::Suspense`] if the awaited resources are in fact already available.
///
/// To not load asynchronous content on the server, it is better to instead inject a fake resolver (e.g. an HTTP client service) that does not store a [`Waker`](`core::task::Waker`).
pub trait ContentRuntime {
	/// Schedules a [`ContentFuture`] to be evaluated to completion.
	///
	/// **The scheduling specifics depend entirely on the [`ContentRuntime`] implementation.**
	/// In particular, in extreme cases, it may be blocking or discard `content_future` immediately.
	///
	/// More commonly, it will likely do some initial polling synchronously.
	///
	/// For best compatibility, callers of this method should take these possibilities into account.
	/// However, it is possible (and often sensible) for a component to be compatible with only a certain subset of possible scheduler behaviour.
	fn start_content_future(&self, content_future: ContentFuture);
}
derive_dependency!(dyn ContentRuntime);

impl<F: Fn(ContentFuture)> ContentRuntime for F {
	fn start_content_future(&self, content_future: ContentFuture) {
		self(content_future)
	}
}
