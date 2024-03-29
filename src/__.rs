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

pub use lignin_schema;
pub use rhizome;
pub use static_assertions;
pub use try_lazy_init;
pub use typed_builder;

#[cfg(feature = "tracing")]
pub use tracing;

#[cfg(not(feature = "tracing"))]
pub mod tracing {
	pub use asteracea_proc_macro_definitions::{empty_block as error, fake_span as debug_span};

	pub struct Span;
	impl Span {
		#[allow(clippy::must_use_unit)]
		#[must_use]
		pub fn entered(self) {
			drop(self)
		}
	}
}

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

/// Used to duck-type [`tracing::Value`] implementations on component parameters,
/// via [autoderef-specialisation](https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html),
/// for use in [`mod@tracing::instrument`]'s `fields` argument.
///
/// # Example
///
/// ```rust
/// #[derive(Debug)]
/// struct YesDebug;
/// struct NoDebug;
///
/// //FIXME: `debug_span` isn't entirely hygienic, so the parameter can't be called `debug`.
/// // See <https://github.com/tokio-rs/tracing/issues/1318>.
/// pub fn auto_values(value: u32, debug_: YesDebug, neither: NoDebug) {
///     let _span = ::asteracea::__::tracing::debug_span!(
///             "auto_values",
///             value = {
///                 use ::asteracea::__::CoerceTracingValue;
///                 (&&&&&::asteracea::__::InertWrapper(&value)).coerce()
///             },
///             debug = {
///                 use ::asteracea::__::CoerceTracingValue;
///                 (&&&&&::asteracea::__::InertWrapper(&debug_)).coerce()
///             },
///             neither = {
///                 use ::asteracea::__::CoerceTracingValue;
///                 (&&&&&::asteracea::__::InertWrapper(&neither)).coerce()
///             },
///         ).entered();
///     drop((value, debug_, neither))
/// }
/// ```
#[cfg(feature = "tracing")]
pub trait CoerceTracingValue<'a> {
	type CoercedValue: 'a + tracing::Value;
	#[must_use]
	fn coerce(&self) -> Self::CoercedValue;
}

#[cfg(feature = "tracing")]
impl<'a, T: ?Sized> CoerceTracingValue<'a> for &&&&InertWrapper<&'a T>
where
	&'a T: tracing::Value,
{
	type CoercedValue = &'a T;
	fn coerce(&self) -> Self::CoercedValue {
		self.0
	}
}

#[cfg(feature = "tracing")]
impl<'a, T> CoerceTracingValue<'a> for &&&InertWrapper<&'a Option<T>>
where
	&'a T: Debug,
{
	type CoercedValue = Option<tracing::field::DebugValue<&'a T>>;
	fn coerce(&self) -> Self::CoercedValue {
		self.0.as_ref().map(tracing::field::debug)
	}
}

#[cfg(feature = "tracing")]
impl<'a, T: ?Sized> CoerceTracingValue<'a> for &&InertWrapper<&'a T>
where
	&'a T: Debug,
{
	type CoercedValue = tracing::field::DebugValue<&'a T>;
	fn coerce(&self) -> Self::CoercedValue {
		tracing::field::debug(self.0)
	}
}

#[cfg(feature = "tracing")]
impl<T> CoerceTracingValue<'_> for &InertWrapper<&Option<T>> {
	type CoercedValue = Option<tracing::field::DebugValue<NotValueNotDebugDebug>>;
	fn coerce(&self) -> Self::CoercedValue {
		self.0
			.as_ref()
			.map(|_| tracing::field::debug(NotValueNotDebugDebug))
	}
}

#[cfg(feature = "tracing")]
impl<T: ?Sized> CoerceTracingValue<'_> for InertWrapper<&T> {
	type CoercedValue = tracing::field::DebugValue<NotValueNotDebugDebug>;
	fn coerce(&self) -> Self::CoercedValue {
		tracing::field::debug(NotValueNotDebugDebug)
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
