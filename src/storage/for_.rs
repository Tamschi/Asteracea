use std::pin::Pin;

use linotype::Linotype;

use crate::error::Escalation;

/// Storage for [`for`](`For`) expressions.
pub struct For<'a, Storage, Q: ToOwned> {
	linotype: Pin<Linotype<Q::Owned, Storage>>,
	factory: Box<dyn 'a + FnMut() -> Result<Storage, Escalation>>,
}

impl<'a, Storage, Q: ToOwned> For<'a, Storage, Q> {
	pub fn new(factory: impl 'static + FnMut() -> Result<Storage, Escalation>) -> Self {
		Self::new_(Box::new(factory))
	}

	fn new_(factory: Box<dyn 'static + FnMut() -> Result<Storage, Escalation>>) -> Self {
		Self {
			linotype: Linotype::new().pin(),
			factory,
		}
	}
}
