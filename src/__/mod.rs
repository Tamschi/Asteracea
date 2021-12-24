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
