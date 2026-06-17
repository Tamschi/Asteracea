use core::fmt::Debug;
use lignin::CallbackRegistration;
use std::{
	fmt::{self, Formatter},
	mem::ManuallyDrop,
	pin::Pin,
	rc::Rc,
	sync::Arc,
};
use try_lazy_init::Lazy;

pub use bon;
pub use core;
pub use lignin_schema;
pub use pinned_init;
pub use rhizome;
pub use static_assertions;
pub use try_lazy_init;

pub use asteracea_proc_macro_definitions::components;

pub use super::{Component, Substrate};

/// Only implemented for functions that have a signature ABI-compatible with `fn(*const R, T)`!
/// See `event_binding.rs` is the asteracea_proc-macro-definition crate for more information.
pub trait CallbackHandler<R: ?Sized, T, Disambiguate> {}
impl<R: ?Sized, T, F> CallbackHandler<R, T, *const R> for F where F: FnOnce(*const R, T) {}
impl<R: ?Sized, T, F> CallbackHandler<R, T, &'static R> for F where F: FnOnce(&R, T) {}
impl<R: ?Sized, T, F> CallbackHandler<R, T, Pin<&'static R>> for F where F: FnOnce(Pin<&R>, T) {}

// Clippy complains about the type complexity of this if it appears directly as component field.
pub type DroppableLazyCallbackRegistration<Component, ParameterFn> =
	ManuallyDrop<Lazy<CallbackRegistration<Component, ParameterFn>>>;

/// Automatically instantiates a [`Built::Builder`] for a type [`B: Built`](`Built`)
/// that can be inferred from a phantom array.
pub fn infer_builder<B: Built>(_phantom: [B; 0]) -> B::Builder {
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

pub struct NotValueNotDebugDebug;
impl Debug for NotValueNotDebugDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str("(`!tracing::Value + !Debug`)")
	}
}

/// A newtype that does absolutely nothing by itself.
///
/// This is needed to de-collide [`CoerceTracingValue`] due to [`impl<T: ?Sized + Debug> Debug for &T`](https://doc.rust-lang.org/stable/core/fmt/trait.Debug.html#implementors),
/// for example.
pub struct InertWrapper<T: ?Sized>(pub T);

pub trait UnBorrow {
	type Target: ?Sized;

	fn one_borrow(&self) -> &Self::Target;
}

impl<T: ?Sized + UnBorrow> UnBorrow for &T {
	type Target = T::Target;

	fn one_borrow(&self) -> &Self::Target {
		<T as UnBorrow>::one_borrow(self)
	}
}

impl<T: ?Sized + UnBorrow> UnBorrow for &mut T {
	type Target = T::Target;

	fn one_borrow(&self) -> &Self::Target {
		<T as UnBorrow>::one_borrow(self)
	}
}

macro_rules! impl_un_borrow {
	($($type:ty),*$(,)?) => {$(
		impl UnBorrow for $type {
			type Target = Self;

			fn one_borrow(&self) -> &Self::Target {
				self
			}
		}
	)*};
}

impl_un_borrow!(char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, str, String, usize, isize);

impl<T: ?Sized> UnBorrow for Box<T> {
	type Target = Self;

	fn one_borrow(&self) -> &Self::Target {
		self
	}
}

impl<T: ?Sized> UnBorrow for Rc<T> {
	type Target = Self;

	fn one_borrow(&self) -> &Self::Target {
		self
	}
}

impl<T: ?Sized> UnBorrow for Arc<T> {
	type Target = Self;

	fn one_borrow(&self) -> &Self::Target {
		self
	}
}
