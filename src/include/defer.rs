use crate::error::{Escalate, Escalation};
use std::{
	error::Error,
	fmt::{self, Display, Formatter},
	pin::Pin,
	result::Result,
};
use try_lazy_init::LazyTransform;

/// Storage for [`defer`](`Defer`) expressions.
pub struct Defer<'a, Storage>(
	LazyTransform<Box<dyn 'a + FnOnce() -> Result<Storage, Escalation>>, Storage>,
);
impl<'a, Storage> Defer<'a, Storage> {
	/// Creates a new [`Defer<Storage>`] instance storing the specified constructor for later use.
	pub fn new(
		deferred_constructor: impl 'static + FnOnce() -> Result<Storage, Escalation>,
	) -> Self {
		Self(LazyTransform::new(Box::new(deferred_constructor)))
	}

	/// Retrieves a reference to the constructed `Storage`, constructing it if necessary.
	///
	/// # Errors
	///
	/// Iff construction fails, that [`Escalation`] is returned verbatim.
	///
	/// Iff construction failed previously, a less specific [`Escalation`] is returned without further attempts.
	pub fn get_or_poison(self: Pin<&Self>) -> Result<Pin<&Storage>, Escalation> {
		#[derive(Debug)]
		struct DeferredSubexpressionConstructorFailedPreviously;
		impl Error for DeferredSubexpressionConstructorFailedPreviously {}
		impl Display for DeferredSubexpressionConstructorFailedPreviously {
			fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
				f.write_str("Deferred (`defer` sub-expression) constructor failed previously.")
			}
		}

		self.0
			.get_or_create_or_poison(|deferred| deferred())
			.map_err(|first_time_error| {
				first_time_error
					.unwrap_or_else(|| DeferredSubexpressionConstructorFailedPreviously.escalate())
			})
			.map(|storage| unsafe {
				//SAFETY: Derived in place from `Pin<&Self>`.
				Pin::new_unchecked(&*(storage as *const _))
			})
	}

	/// Retrieves a reference to the constructed `Storage`, iff one is available already.
	#[must_use]
	pub fn get(self: Pin<&Self>) -> Option<Pin<&Storage>> {
		self.0
			.get()
			.map(|storage| unsafe { Pin::new_unchecked(&*(storage as *const _)) })
	}
}
