use super::dependency_injection::ResourceNode;
use super::private::Dereferenceable;
use crate::error::{
	EscalateResult, Escalation, IncompatibleRuntimeDependency, RuntimeDependencyMissing,
};
use crate::services::{EventHandlerRuntime, ServiceHandle};
use core::future::Future;
use lignin::web::Event;
use lignin::{CallbackRegistration, ThreadSafe};
use rhizome::sync::Extract;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// A [`Future`] representing a triggered asynchronous event handler.
pub type EventHandlerFuture = Pin<Box<dyn Future<Output = ()>>>;

pub struct AsyncEventBinding<Owner: ?Sized> {
	runtime: ServiceHandle<dyn EventHandlerRuntime>,
	registration: Mutex<Option<CallbackRegistration<ThreadSafe, fn(Event)>>>,
	///TODO:
	/// Not optimised. The outer `Mutex<Option<Pin<Arc<>>>>` can be replaced with an `AtomicPtr<>` here.
	/// Doing so incurs a sporadic double-allocation of the bounce pad, but that's still more efficient than locking the [`Mutex`] on every render.
	back_reference: Mutex<Option<Pin<Arc<Mutex<Option<Pin<Dereferenceable<Owner>>>>>>>>,
}
impl<Owner: ?Sized> AsyncEventBinding<Owner> {
	pub fn new(resource_node: Pin<&ResourceNode>) -> Result<Self, Escalation> {
		Ok(Self {
			runtime: <dyn EventHandlerRuntime>::extract(resource_node)
				.map_err(IncompatibleRuntimeDependency::<dyn EventHandlerRuntime>::new_and_log)
				.escalate()?
				.ok_or_else(RuntimeDependencyMissing::<dyn EventHandlerRuntime>::new_and_log)
				.escalate()?,
			registration: None.into(),
			back_reference: None.into(),
		})
	}
}

pub trait AsyncEventHandler<Owner: ?Sized, Event> {
	fn instantiate(
		&self,
		owner: Pin<Arc<Mutex<Option<Pin<Dereferenceable<Owner>>>>>>,
		event: Event,
	) -> EventHandlerFuture;
}
