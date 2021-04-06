//! # Features
//!
//! ## `"backtrace"`
//!
//! Enables additional error traces, at the cost of code size and performance.
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

#![doc(html_root_url = "https://docs.rs/asteracea/0.0.2")]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::match_bool)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::redundant_else)]

pub use asteracea_proc_macro_definitions::{bump_format, component, fragment, trace_escalations};
pub use bumpalo;
pub use lignin;
pub use rhizome;

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

pub mod error;

#[cfg(feature = "services")]
pub mod services;

/// Types implementing this trait can be accepted as value by conditional attributes.
///
/// It's recommended to only implement this trait if the conversion is very cheap.
pub trait ConditionalAttributeValue<'a> {
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
