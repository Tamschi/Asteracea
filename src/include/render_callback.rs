//! (Mostly) type-erased render delegate types.

use crate::error::Result;
use bumpalo::Bump;
use lignin::Node;

/// A render callback that can be called at most once.
///
/// Inherits an `S` is [`ThreadSafety`] constraint.
pub type RenderOnce<'a, 'bump, S> = dyn 'a + FnOnce(&'bump Bump) -> Result<Node<'bump, S>>;

/// A render callback that can be called any amount of times.
///
/// Inherits an `S` is [`ThreadSafety`] constraint.
pub type RenderMut<'a, 'bump, S> = dyn 'a + FnMut(&'bump Bump) -> Result<Node<'bump, S>>;
