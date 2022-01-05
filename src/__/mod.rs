use lignin::CallbackRegistration;
use std::{mem::ManuallyDrop, pin::Pin};
use try_lazy_init::Lazy;

pub use lignin_schema;
pub use static_assertions;
pub use try_lazy_init;
pub use typed_builder;

/// Only implemented for functions that have a signature ABI-compatible with `fn(*const R, T)`!
/// See `event_binding.rs` is the asteracea_proc-macro-definition crate for more information.
pub trait CallbackHandler<R: ?Sized, T, Disambiguate> {}
impl<R: ?Sized, T, F> CallbackHandler<R, T, *const R> for F where F: FnOnce(*const R, T) {}
impl<R: ?Sized, T, F> CallbackHandler<R, T, &'static R> for F where F: FnOnce(&R, T) {}
impl<R: ?Sized, T, F> CallbackHandler<R, T, Pin<&'static R>> for F where F: FnOnce(Pin<&R>, T) {}

// Clippy complains about the type complexity of this if it appears directly as component field.
pub type DroppableLazyCallbackRegistration<Component, ParameterFn> =
	ManuallyDrop<Lazy<CallbackRegistration<Component, ParameterFn>>>;

/// Automatically instantiates as [`Built::Builder`] for a type [`B: Built`](`Built`)
/// that can be inferred from a phantom array.
pub fn infer_builder<B: Built>(phantom: [B; 0]) -> B::Builder {
	drop(phantom);
	B::builder()
}

/// A buildable type.
pub trait Built {
	type Builder;
	fn builder() -> Self::Builder;
}

/// FIXME: Properly support custom parent parameters.
pub struct AnonymousContentParentParameters {}
/// FIXME: Properly support custom parent parameters.

pub struct AnonymousContentParentParametersBuilder;

impl Built for AnonymousContentParentParameters {
	type Builder = AnonymousContentParentParametersBuilder;

	#[must_use]
	fn builder() -> Self::Builder {
		AnonymousContentParentParametersBuilder
	}
}

impl AnonymousContentParentParametersBuilder {
	#[must_use]
	pub fn build(self) -> AnonymousContentParentParameters {
		let _ = self;
		AnonymousContentParentParameters {}
	}
}
