use super::dependency_injection::ResourceNode;
use super::private::Dereferenceable;
use crate::error::{
	EscalateResult, Escalation, IncompatibleRuntimeDependency, RuntimeDependencyMissing,
};
use crate::services::{EventHandlerRuntime, ServiceHandle};
use core::future::Future;
use lignin::web::Event;
use lignin::{CallbackRef, CallbackRegistration, ThreadSafe};
use rhizome::sync::Extract;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

/// A [`Future`] representing a triggered asynchronous event handler.
pub type EventHandlerFuture = Pin<Box<dyn 'static + Send + Future<Output = ()>>>;

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

	/// # Safety
	///
	/// This method must, for every one `self`, always be called with the same `owner`.
	/// `self` must be dropped before `owner` is.
	pub unsafe fn render(&self, owner: Pin<&Owner>) -> CallbackRef<ThreadSafe, fn(Event)>
	where
		Owner: Sync,
	{
		let mut registration = self.registration.lock().unwrap();
		if registration.is_none() {
			let mut back_reference = self.back_reference.lock().unwrap();
			*back_reference = Some(Arc::pin(Mutex::new(Some(unsafe {
				Pin::new_unchecked(Dereferenceable::new(NonNull::new_unchecked(
					owner.get_ref() as *const _ as *mut _,
				)))
			}))));

			todo!()
		}

		registration.as_ref().unwrap().to_ref()
	}
}

impl<Owner: ?Sized> Drop for AsyncEventBinding<Owner> {
	fn drop(&mut self) {
		if let Some(back_reference) = self.back_reference.lock().unwrap().as_ref() {
			*back_reference.lock().unwrap() = None;
		}
	}
}

pub trait AsyncEventHandler<Owner: ?Sized, Event> {
	fn instantiate(
		&self,
		owner: Pin<Arc<Mutex<Option<Pin<Dereferenceable<Owner>>>>>>,
		event: Event,
	) -> EventHandlerFuture;
}
