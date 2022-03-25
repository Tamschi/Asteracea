//! (Mostly) type-erased render delegate types.

use crate::error::Result;
use bumpalo::Bump;
use lignin::{Guard, ThreadSafety};

/// A render callback that can be called at most once.
///
/// Inherits an `S` is [`ThreadSafety`] constraint.
pub type RenderOnce<'a, 'bump, S> = dyn 'a + FnOnce(&'bump Bump) -> Result<Guard<'bump, S>>;

/// A render callback that can be called any amount of times.
///
/// Inherits an `S` is [`ThreadSafety`] constraint.
pub type RenderMut<'a, 'bump, S> = dyn 'a + FnMut(&'bump Bump) -> Result<Guard<'bump, S>>;

mod sealed {
	use lignin::ThreadSafety;

	use super::{RenderMut, RenderOnce};

	pub trait Sealed {}
	impl<S: ThreadSafety> Sealed for RenderOnce<'_, '_, S> {}
	impl<S: ThreadSafety> Sealed for RenderMut<'_, '_, S> {}
}
use sealed::Sealed;

/// An unspecific thread render callback.
///
/// This is mainly used to constrain meta implementations in this crate *as a hint* and shouldn't be minded by component implementations
/// (which should expect [`RenderOnce`] or [`RenderMut`] directly instead).
pub trait RenderCallback: Sealed {}
impl<'bump, S: ThreadSafety> RenderCallback for RenderOnce<'_, 'bump, S> {}
impl<'bump, S: ThreadSafety> RenderCallback for RenderMut<'_, 'bump, S> {}
