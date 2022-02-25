use crate::services::ServiceHandle;
use futures_core::Future;
use rhizome::sync::derive_dependency;
use std::{
	pin::Pin,
	task::{Context, Poll},
};

/// Call [`.invalidate()`](`dyn Invalidator::invalidate`) to request a re-render of the injected site.
pub trait Invalidator {
	/// Requests a re-render of the injected site, while passing along a context that is to be woken
	/// once the updated GUI is (sure to be) presented to the user.
	///
	/// Unlike the [`Future`] API, this one is *not lazy*. (Note the [unit](`()`) return type.)  
	/// It also does not do the state tracking that is necessary to yield in an an `async` context.
	///
	/// In most cases, a consumer will call [`.next_frame().await`](`dyn Invalidator::next_frame`) instead, which has those properties.
	///
	/// > The re-render *should* happen, generally sooner rather than later, but it is not entirely guaranteed.
	fn invalidate_with_context(&self, on_presented: Option<&mut Context<'_>>);
}
derive_dependency!(dyn Invalidator);

impl<F: Fn(Option<&mut Context<'_>>)> Invalidator for F {
	fn invalidate_with_context(&self, on_presented: Option<&mut Context<'_>>) {
		self(on_presented)
	}
}

const _: () = {
	fn assert_usability(
		handle: &ServiceHandle<dyn Invalidator>,
	) -> &(dyn 'static + Send + Sync + std::any::Any) {
		handle
	}
};

impl dyn Invalidator {
	/// Requests a re-render of the injected site.
	///
	/// > The re-render *should* happen, generally sooner rather than later, but it is not entirely guaranteed.
	pub fn invalidate(&self) {
		self.invalidate_with_context(None)
	}

	/// Constructs a [`Future`] that can be used to wait past a user-visible GUI update.
	///
	/// This API follows Rust `async` semantics and as such is lazy:
	/// **A re-render will not be requested until the resulting [`NextFrame`] is polled.
	///
	/// <!-- TODO: This should have an example showing an asynchronous event handler workflow, once that is available. -->
	pub fn next_frame(&self) -> NextFrame<'_> {
		NextFrame(Some(self))
	}
}

/// Returned from [`dyn Invalidator::next_frame`].
///
/// [Await](https://doc.rust-lang.org/stable/std/keyword.await.html) this [`Future`] to request a re-render and wait for it to be presented to the user.
#[must_use = "`Invalidator::next_frame` is lazy: A re-render is only requested iff this resulting `Future` is polled."]
pub struct NextFrame<'a>(Option<&'a dyn Invalidator>);

impl Future for NextFrame<'_> {
	type Output = ();

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		match self.0.take() {
			Some(invalidator) => {
				invalidator.invalidate_with_context(Some(cx));
				Poll::Pending
			}
			None => Poll::Ready(()),
		}
	}
}
