//! Runtime components used by generated Asteracea components.
//!
//! This is not 'a runtime' in so far as that it's not overarching code in any shape or form, and most of it isn't required for many components.
//! Instead, these are smaller building blocks to save on duplicate generated code, and to make it easier to hand-write components should that be required.
//!
//! See the individual item documentation for which of Asteracea's grammar features requires which of them.

mod defer;
#[doc(hidden)]
pub mod for_;

pub use defer::Defer;
pub use for_::For;
