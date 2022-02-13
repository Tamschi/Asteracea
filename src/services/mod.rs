//! Injectable services.

use rhizome::sync::{RefExtract, RefExtracted};
use std::any::TypeId;

mod content_runtime;
mod invalidator;

pub use content_runtime::ContentRuntime;
pub use invalidator::Invalidator;

pub type ServiceHandle<Service> = RefExtracted<TypeId, <Service as RefExtract>::ExtractedTarget>;
