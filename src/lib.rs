#![cfg_attr(not(any(feature = "topiary", feature = "rhizome")), no_std)]
#![doc(html_root_url = "https://docs.rs/asteracea/0.0.2")]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub use asteracea_proc_macro_definitions::{bump_format, component, fragment};
pub use lazy_static;
pub use lignin_schema;

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

#[cfg(feature = "rhizome")]
pub use rhizome_crate as rhizome;

#[cfg(feature = "rhizome")]
pub mod extractable_resolution_error;

#[cfg(feature = "services")]
pub mod services;

#[cfg(feature = "styles")]
pub mod styles;

#[cfg(feature = "topiary")]
pub use topiary_crate as topiary;
