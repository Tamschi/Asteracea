use crate::include::async_event_binding::EventHandlerFuture;
use rhizome::sync::derive_dependency;

/// A resource used by [`AsyncEventBinding`](`crate::include::async_event_binding::AsyncEventBinding`)s
/// to schedule [`EventHandlerFuture`]s.
pub trait EventHandlerRuntime {
	/// Schedules an [`EventHandlerFuture`] to be evaluated to completion.
	fn start_event_handler_future(&self, event_handler_future: EventHandlerFuture);
}
derive_dependency!(dyn EventHandlerRuntime);

impl<F: Fn(EventHandlerFuture)> EventHandlerRuntime for F {
	fn start_event_handler_future(&self, event_handler_future: EventHandlerFuture) {
		self(event_handler_future)
	}
}
