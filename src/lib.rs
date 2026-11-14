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
//TODO: Reorganise some of these into a module.
pub use traits::{
	AsyncInit, ContentParent, ContentParentOf, ContentParentSubstrate,
	DefaultAccumulationSubstrate, Init, Render, Substrate,
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
