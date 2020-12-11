#![doc(html_root_url = "https://docs.rs/asteracea/0.0.2")]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::match_bool)]

pub use asteracea_proc_macro_definitions::{bump_format, component, fragment};
pub use lazy_static;
pub use lignin_schema;
pub use rhizome;
pub use typed_builder;

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
