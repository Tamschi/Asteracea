#![doc(html_root_url = "https://docs.rs/asteracea/0.0.2")]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::sync::Arc;

pub use asteracea_proc_macro_definitions::{bump_format, component, fragment};
use error::ExtractableResolutionError;
pub use lazy_static;
pub use lignin_schema;
use lignin_schema::lignin::bumpalo::Bump;
pub use rhizome;
pub use typed_builder;

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

pub mod error;

#[cfg(feature = "services")]
pub mod services;
