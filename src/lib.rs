#![doc(html_root_url = "https://docs.rs/asteracea/0.0.2")]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

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
