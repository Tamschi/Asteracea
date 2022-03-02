use super::dependency_injection::ResourceNode;
use super::private::Dereferenceable;
use crate::error::{
	EscalateResult, Escalation, IncompatibleRuntimeDependency, RuntimeDependencyMissing,
};
use crate::services::{EventHandlerRuntime, ServiceHandle};
use core::future::Future;
use core::ptr;
use lignin::web::Event;
use lignin::{CallbackRef, CallbackRegistration, ThreadSafe};
use rhizome::sync::Extract;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

/// A [`Future`] representing a triggered asynchronous event handler.
pub type EventHandlerFuture = Pin<Box<dyn 'static + Send + Future<Output = ()>>>;

pub struct AsyncEventBinding<Owner: ?Sized> {
	runtime: ServiceHandle<dyn EventHandlerRuntime>,
	registration: Mutex<Option<CallbackRegistration<Self, fn(Event)>>>,
	/// This [`AtomicPtr<_>`] replaces a `Mutex<Option<Pin<Arc<_>>>>`.
	/// It's less unwieldy and just a little faster too. (Synchronised externally by `self.registration`.)
	back_reference: AtomicPtr<Mutex<Option<Pin<Dereferenceable<Owner>>>>>,
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
			back_reference: AtomicPtr::new(ptr::null_mut()),
		})
	}

	/// # Safety
	///
	/// This method must, for every one `self`, always be called with the same `owner`.
	/// `self` must be dropped before `owner` is.
	pub unsafe fn render(self: Pin<&Self>, owner: Pin<&Owner>) -> CallbackRef<ThreadSafe, fn(Event)>
	where
		Owner: Sync,
	{
		let mut registration = self.registration.lock().unwrap();
		if registration.is_none() {
			let back_reference = Arc::new(Mutex::new(Some(unsafe {
				Pin::new_unchecked(Dereferenceable::new(NonNull::new_unchecked(
					owner.get_ref() as *const _ as *mut Owner,
				)))
			})));
			self.back_reference.store(
				Arc::into_raw(Arc::clone(&back_reference)) as *mut _,
				Ordering::Relaxed,
			);

			*registration = Some(CallbackRegistration::<_, fn(Event)>::new(
				self,
				launch_handler,
			))
		}

		registration.as_ref().unwrap().to_ref()
	}
}

impl<Owner: ?Sized> Drop for AsyncEventBinding<Owner> {
	fn drop(&mut self) {
		// The registration must be dropped first, to ensure that `launch_handler` has exited.
		*self.registration.get_mut().unwrap() = None;
		if let Some(back_reference) =
			unsafe { self.back_reference.load(Ordering::Relaxed).as_ref() }
		{
			*back_reference.lock().unwrap() = None;
		}
	}
}

fn launch_handler<Owner: ?Sized>(
	async_event_binding: *const AsyncEventBinding<Owner>,
	event: Event,
) {
	todo!()
}

pub trait AsyncEventHandler<Owner: ?Sized, Event> {
	fn instantiate(
		&self,
		owner: Pin<Arc<Mutex<Option<Pin<Dereferenceable<Owner>>>>>>,
		event: Event,
	) -> EventHandlerFuture;
}
