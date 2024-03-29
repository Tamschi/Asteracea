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

pub use asteracea_proc_macro_definitions::{bump_format, component, fragment};
pub use bumpalo;
pub use lignin;
pub use try_lazy_init;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub mod components;
pub mod error;
pub mod include;
pub mod services;

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
