//! A lightweight but flexible web frontend framework in Rust.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252FAsteracea)](https://iteration-square.schichler.dev/#narrow/stream/project.2FAsteracea)
//!
//! # Features
//!
//! ## `"error-abort"`
//!
//! Reserved. Will be used to abort-the process on GUI error escalation.
//!
//! ## `"force-unwind"`
//!
//! Force the use of panics for `Escalation` propagation. This may improve code size and app performance.
//!
//! ## `"services"`
//!
//! TODO
//!
//! ## `"tracing"`
//!
//! Enables [`tracing`](https://docs.rs/tracing/0.1/tracing/) instrumentation of `::new` and `.render` functions
//! (on components generated using [`component`]).
//!
//! **Note:** This currently requires `tracing = { version = "0.1", default-features = false }` in your dependencies to resolve this crate.

#![doc(html_root_url = "https://docs.rs/asteracea/0.0.2")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::match_bool)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::redundant_else)]
#![allow(clippy::semicolon_if_nothing_returned)]

//FIXME: This won't be necessary anymore once `$crate` is in use everywhere.
extern crate self as asteracea;

#[macro_export]
macro_rules! components {
	// Pass `$crate` to the procedural macro in a way that doesn't require parsing it.
	($($tt:tt)*) => {
		$crate::__::components!([$crate::__] $($tt)*);
	};
}

use std::ops::{Add, BitOr};

pub use bumpalo;
pub use lignin;
pub use try_lazy_init;

mod traits;
//TODO: Reorganise some of these (component-traits, substrate-traits) into a module.
pub use traits::{
	AsyncInit, ContentParent, ContentParentOf, ContentParentSubstrate,
	DefaultAccumulationSubstrate, Init, Render, SignalSubstrate, SignalSubstrateCellCyclic,
	SignalSubstrateComputed, SignalSubstrateComputedUncached, SignalSubstrateComputedUncachedMut,
	SignalSubstrateDistinct, SimpleContentSubstrate, Substrate,
};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

//pub mod components; //TODO
pub mod include;
// pub mod services; //TODO

/// Types implementing this trait can be accepted as value by conditional attributes.
///
/// It's recommended to only implement this trait if the conversion is very cheap.
pub trait ConditionalAttributeValue<'a> {
	/// Borrows the attribute value as representative [`Option<&str>`].
	fn into_str_option(self) -> Option<&'a str>;
}

impl ConditionalAttributeValue<'static> for bool {
	#[inline]
	fn into_str_option(self) -> Option<&'static str> {
		match self {
			false => None,
			true => Some(""),
		}
	}
}

impl<'a> ConditionalAttributeValue<'a> for Option<&'a str> {
	#[inline]
	fn into_str_option(self) -> Self {
		self
	}
}

#[allow(non_snake_case)]
#[doc(hidden)]
pub mod __;

/// Provided to [`ContentParent::start_accumulation`] (and [`DefaultAccumulationSubstrate::default_start_accumulation`])
/// with the semantics of [`Iterator::size_hint`].
///
/// This struct implements operators to make macro code more manageable.
pub struct SizeHint(pub usize, pub Option<usize>);

impl SizeHint {
	/// Use this only when a more precise determination isn't feasible.
	pub const UNKNOWN: Self = Self(0, None);

	pub const ZERO: Self = Self(0, Some(0));
	pub const ONE: Self = Self(1, Some(1));
}

/// Bounds-wise [`usize::saturating_add`].
impl Add for SizeHint {
	type Output = Self;

	#[inline(always)] //FIXME: `const`
	fn add(self, rhs: Self) -> Self::Output {
		Self(
			self.0.saturating_add(rhs.0),
			self.1.zip(rhs.1).map(|(a, b)| a.saturating_add(b)),
		)
	}
}

/// Range union.
impl BitOr for SizeHint {
	type Output = Self;

	#[inline(always)] //FIXME: `const`
	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0.min(rhs.0), self.1.zip(rhs.1).map(|(a, b)| a.max(b)))
	}
}

#[doc(inline)]
/// Macros that represent the modalities of Asteracea `let` statements,
/// which persist values in [`Init::init`] or [`AsyncInit::init`] for
/// use also in [`Render::render`].
///
/// Depending on the macro used, the value is accessible in different ways:
///
/// ```rust
/// # use std::marker::PhantomPinned;
/// #
/// # struct Pinned {
/// # 	_phantom: PhantomPinned
/// # }
/// #
/// # impl Pinned {
/// # 	fn new() -> Self {
/// # 		Self { _phantom: PhantomPinned }
/// # 	}
/// #
/// # 	fn when_pinned(self: Pin<&Self>) {
/// # 		let _ = self;
/// # 	}
/// #
/// # 	fn when_pinned_mut(self: Pin<&mut Self>) {
/// # 	}
/// # }
/// #
/// asteracea::components! {
/// 	#![substrate = Sub]
///
/// 	Test => {
/// 		let a = pin!(Pinned::new());
/// 		let mut b = pin!(Pinned::new());
///
/// 		({
/// 			// In `init`.
/// 			a.when_pinned();
/// 			b.when_pinned();
/// 			b.as_mut().when_pinned_mut();
/// 		})
/// 		[{
/// 			// In `render`.
/// 			a.when_pinned();
/// 			b.when_pinned();
/// 			b.as_mut().when_pinned_mut();
/// 		}]
/// 	}
/// }
/// ```
///
/// To create plain bindings and variables in `init` or `render`,
/// instead use `(let) …;` or `[let] …;` statements, respectively,
/// with usual Rust syntax.
pub use __::let_macros::value as persisted_let;

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_type {
	($type:ty, $substrate:ty,) => {
		$type
	};
}

#[doc(hidden)]
#[macro_export]
/// Stored verbatim. Access through <code>[Pin](`std::pin::Pin`)&lt;&T></code> or <code>[Pin](`std::pin::Pin`)&lt;&mut T></code>.
macro_rules! __pin_value {
	(($($value:tt)*), $substrate:ty,) => {
		$($value)*
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_access {
	($ptr:expr, $substrate:ty, $guarding_borrow:expr,,) => {
		unsafe { $crate::__::Pin::new_unchecked($crate::__::co_borrow($guarding_borrow, $ptr)) }
	};
	($ptr:expr, $substrate:ty, $guarding_borrow:expr, mut,) => {
		unsafe { $crate::__::Pin::new_unchecked($crate::__::co_borrow_mut($guarding_borrow, $ptr)) }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cell_type {
	($type:ty, $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrate<$type>>::SignalArcDynCell
	};
}

#[doc(hidden)]
#[macro_export]
/// Calls [`SignalSubstrate::cell`] once.\
/// Access through <code>&[SignalSubstrate](`SignalSubstrate`)&lt;T>::[SignalDynCell](`SignalSubstrate::SignalDynCell`)</code>.
macro_rules! __cell_value {
	(($($value:tt)*), $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrate<_>>::cell($($value)*)
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cell_access {
	($ptr:expr, $substrate:ty, $guarding_borrow:expr,,) => {
		unsafe { $crate::__::co_borrow($guarding_borrow, $ptr) }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cell_cyclic_type {
	($type:ty, $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrate<$type>>::SignalArcDynCell
	};
}

#[doc(hidden)]
#[macro_export]
/// Calls [`SignalSubstrateCellCyclic::cell_cyclic`] once.\
/// Access through <code>&[SignalSubstrate](`SignalSubstrate`)&lt;T>::[SignalDynCell](`SignalSubstrate::SignalDynCell`)</code>.
macro_rules! __cell_cyclic_value {
	(($($value:tt)*), $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrateCellCyclic<_, _>>::cell_cyclic($($value)*)
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cell_cyclic_access {
	($ptr:expr, $substrate:ty, $guarding_borrow:expr,,) => {
		unsafe { $crate::__::co_borrow($guarding_borrow, $ptr) }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __computed_type {
	($type:ty, $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrate<$type>>::SignalArcDyn
	};
}

#[doc(hidden)]
#[macro_export]
/// Calls [`SignalSubstrateComputed::computed`] once.\
/// Access through <code>&[SignalSubstrate](`SignalSubstrate`)&lt;T>::[SignalDyn](`SignalSubstrate::SignalDyn`)</code>.
macro_rules! __computed_value {
	(($($value:tt)*), $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrateComputed<_, _>>::computed(
			$crate::__extract_dot_use!($($value)*)
		)
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __computed_access {
	($ptr:expr, $substrate:ty, $guarding_borrow:expr,,) => {
		unsafe { $crate::__::co_borrow($guarding_borrow, $ptr) }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __computed_uncached_type {
	($type:ty, $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrate<$type>>::SignalArcDyn
	};
}

#[doc(hidden)]
#[macro_export]
/// Calls [`SignalSubstrateComputedUncached::computed_uncached`] once.\
/// Access through <code>&[SignalSubstrate](`SignalSubstrate`)&lt;T>::[SignalDyn](`SignalSubstrate::SignalDyn`)</code>.
macro_rules! __computed_uncached_value {
	(($($value:tt)*), $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrateComputedUncached<_, _>>::computed_uncached(
			$crate::__extract_dot_use!($($value)*)
		)
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __computed_uncached_access {
	($ptr:expr, $substrate:ty, $guarding_borrow:expr,,) => {
		unsafe { $crate::__::co_borrow($guarding_borrow, $ptr) }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __computed_uncached_mut_type {
	($type:ty, $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrate<$type>>::SignalArcDyn
	};
}

#[doc(hidden)]
#[macro_export]
/// Calls [`SignalSubstrateComputedUncachedMut::computed_uncached_mut`] once.\
/// Access through <code>&[SignalSubstrate](`SignalSubstrate`)&lt;T>::[SignalDyn](`SignalSubstrate::SignalDyn`)</code>.
macro_rules! __computed_uncached_mut_value {
	(($($value:tt)*), $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrateComputedUncachedMut<_, _>>::computed_uncached_mut(
			$crate::__extract_dot_use!($($value)*)
		)
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __computed_uncached_mut_access {
	($ptr:expr, $substrate:ty, $guarding_borrow:expr,,) => {
		unsafe { $crate::__::co_borrow($guarding_borrow, $ptr) }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __distinct_type {
	($type:ty, $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrate<$type>>::SignalArcDyn
	};
}

#[doc(hidden)]
#[macro_export]
/// Calls [`SignalSubstrateDistinct::distinct`] once.\
/// Access through <code>&[SignalSubstrate](`SignalSubstrate`)&lt;T>::[SignalDyn](`SignalSubstrate::SignalDyn`)</code>.
macro_rules! __distinct_value {
	(($($value:tt)*), $substrate:ty,) => {
		<$substrate as $crate::SignalSubstrateDistinct<_, _>>::distinct(
			$crate::__extract_dot_use!($($value)*)
		)
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __distinct_access {
	($ptr:expr, $substrate:ty, $guarding_borrow:expr,,) => {
		unsafe { $crate::__::co_borrow($guarding_borrow, $ptr) }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __extract_dot_use {
	// Pass `$crate` to the procedural macro in a way that doesn't require parsing it.
	($($tt:tt)*) => {
		$crate::__::extract_dot_use!([$crate::__] $($tt)*)
	};
}
