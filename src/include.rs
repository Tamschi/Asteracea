//! Runtime components used by generated Asteracea components.
//!
//! This is not 'a runtime' in so far as that it's not overarching code in any shape or form, and most of it isn't required for many components.
//! Instead, these are smaller building blocks to save on duplicate generated code, and to make it easier to hand-write components should that be required.
//!
//! See the individual item documentation for which of Asteracea's grammar features requires which of them.

#[doc(hidden)]
pub mod __for_;
pub mod async_;
mod defer;
pub mod render_callback;

pub use __for_::For;
pub use defer::Defer;
