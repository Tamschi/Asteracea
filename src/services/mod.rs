//! Injectable services.

use rhizome::sync::{RefExtract, RefExtracted};
use std::any::TypeId;

mod content_runtime;
mod invalidator;

pub use content_runtime::ContentRuntime;
pub use invalidator::Invalidator;

/// The type of (most) owned handles that are injected when using a trait as dependency key.
///
/// There is no technical requirement that a trait will specify this type as [`Extract::Extracted`](`rhizome::sync::Extract::Extracted`),
/// but it is by far the most straightforward option for owned sharing handles.
pub type ServiceHandle<Service> = RefExtracted<TypeId, <Service as RefExtract>::ExtractedTarget>;
