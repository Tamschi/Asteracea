#![cfg_attr(not(feature = "topiary"), no_std)]
#![forbid(unsafe_code)]

pub use {
    asteracea_proc_macro_definitions::{bump_format, component, fragment},
    lazy_static, lignin_schema,
};

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
