use bon::Builder;
use core::fmt::Debug;
use lignin::CallbackRegistration;
use rhizome::sync::{DynValue, NodeHandle};
use std::{
	any::TypeId,
	fmt::{self, Formatter},
	mem::ManuallyDrop,
	ptr::drop_in_place,
	rc::Rc,
	sync::Arc,
};
use try_lazy_init::Lazy;

pub type DepsNodeHandle = NodeHandle<TypeId, TypeId, DynValue>;
pub use core::{concat, stringify};
pub use eros::ErrorUnion;
pub use inherent_associated_types::Associated;
pub use pinned_init::PinInit;
pub use std::{
	clone::Clone,
	default::Default,
	derive,
	mem::forget,
	ops::Drop,
	pin::Pin,
	result::Result::{self, Err, Ok},
};

pub use bon;
pub use core;
pub use lignin_schema;
pub use pinned_init;
pub use rhizome;
pub use static_assertions;
pub use try_lazy_init;

pub use asteracea_proc_macro_definitions::components;

pub use super::{
	AsyncInit, ContentParent, ContentParentOf, ContentParentSubstrate,
	DefaultAccumulationSubstrate, Init, Render, Substrate,
};

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

struct InitGuard<T: ?Sized> {
	place: *mut T,
}

impl<T: ?Sized> Drop for InitGuard<T> {
	fn drop(&mut self) {
		unsafe { drop_in_place(self.place) };
	}
}

pub fn pinned_init_and_guard<'a, 'i, T: 'a, E>(
	init: impl 'i + PinInit<T, E>,
	place: *mut T,
) -> Result<impl 'a + Drop, E> {
	Ok(unsafe {
		init.__pinned_init(place)?;
		InitGuard { place }
	})
}

#[derive(Builder)]
pub struct NoArgs {}

impl Default for NoArgsBuilder {
	fn default() -> Self {
		NoArgs::builder()
	}
}

#[derive(Builder)]
pub struct NoContent {}

impl Default for NoContentBuilder {
	fn default() -> Self {
		NoContent::builder()
	}
}
