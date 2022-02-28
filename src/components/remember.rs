#![allow(non_snake_case)] // For __Asteracea__anonymous_content

use crate::{
	include::{private::Dereferenceable, render_callback::RenderOnce},
	services::Invalidator,
	__::Built,
};
use bumpalo::Bump;
use lignin::{guard::ConsumedCallback, Guard, Node, ThreadSafety};
use rhizome::sync::Inject;
use std::{
	collections::hash_map::DefaultHasher,
	hash::Hasher,
	marker::PhantomPinned,
	mem,
	ops::Deref,
	pin::Pin,
	sync::{
		atomic::{AtomicBool, AtomicU32, Ordering},
		Arc, Mutex,
	},
	task::Context,
};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct NoParentParameters {}
impl Built for NoParentParameters {
	type Builder = NoParentParametersBuilder<()>;

	fn builder() -> Self::Builder {
		Self::builder()
	}
}

static GLOBAL_COUNTER: AtomicU32 = AtomicU32::new(0);

asteracea::component! {
	/// Tries(!) to skip rendering. `.or_unless` is only evaluated if the content didn't [invalidate](`Invalidator`) itself, and not on first render.
	///
	/// > **Optimize me!**: The current implementation does what it should, but is very inefficient.
	//TODO: Move this generic onto the render method.
	//TODO: Fix this entire file.
	pub Remember<S: ThreadSafety>(
		dyn invalidator?: dyn Invalidator,
	)(
		// for_unchanged?: &R,
		or_unless?: impl FnOnce() -> bool,
		__Asteracea__anonymous_content: (NoParentParameters, Box<RenderOnce<'_, 'bump, S>>),
	) -> Guard<'bump, S>

	new with {
		let this = Arc::pin(Mutex::new(None::<Pin<Dereferenceable<Self>>>));
		if let Some(invalidator) = invalidator {

			//TODO: This is not currently exposed to content children, but needs to be.
			// Should the resource node borrow node be threaded back out with the same lifetime through a `Cow`, as second constructor return value? (Probably yes.)
			<dyn Invalidator>::inject(local_resource_node.borrow(), {
				let this = SafetyHack(Pin::clone(&this));

				//TODO:
				// I'm not certain merging and splitting the control/call flow here is a good idea.
				// There's a good chance that implementing the method separately is better, or maybe the context should just be generally optional at the service side.
				move |context: Option<&mut Context<'_>>| {
					if let Some(this) = this.0.lock().unwrap().as_ref() {
						this.invalidated.store(true, Ordering::Release);
					}
					invalidator.invalidate_with_context(context);
				}
			}).1.ok().expect("Failed to inject new invalidator in `Remember`.");
		}
	}
	let self._pinned = PhantomPinned::default();
	let self.this_set: AtomicBool = false.into();
	let self.this: ClearOnDrop<S> = ClearOnDrop(this);

	let self.invalidated: AtomicBool = true.into();

	// let self.when_unchanged_stored: Mutex<Option<Box<dyn Send + Any>>> = None.into();

	let self.hasher: Mutex<DefaultHasher> = {
		let mut hasher = DefaultHasher::new();
		hasher.write_u32(GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed));
		hasher.into()
	};

	let self.current = Mutex::<Arc::<(Mutex<Bump>, Guard<'static, S>)>>::new(Arc::new((
		Bump::new().into(),
		Guard::new(Node::Multi(&[]), None),
	)));

	{
		if !self.this_set.load(Ordering::Acquire) {
			*self.this.lock().unwrap() = Some(unsafe {
				Pin::new_unchecked(Dereferenceable::new(self.get_ref().into()))
			});
			self.this_set.store(true, Ordering::Release);
		}

		let mut hasher = self.hasher.lock().unwrap();
		let mut current = self.current.lock().unwrap();
		if self.invalidated.swap(false, Ordering::SeqCst)
			|| or_unless.map(|f| f()).unwrap_or_default() {
				let inner_bump = Bump::new();
				let guard = __Asteracea__anonymous_content.1(unsafe { &*(&inner_bump as *const _) })?;

				*current = {
					hasher.write_u8(1);
					Arc::new((
						inner_bump.into(),
						unsafe { detach_guard(guard) },
					))
				};
		}

		unsafe {
			detach_guard(Guard::new(
				Node::Memoized{ state_key: hasher.finish(), content: &*current.1 },
				Some(ConsumedCallback::new(
					|payload_ptr| drop(Arc::from_raw(payload_ptr.cast::<(Mutex<Bump>, Guard<S>)>())),
					Arc::into_raw(Arc::clone(&*current)).cast(),
				)),
			))
		}
	}
}

struct SafetyHack<T>(T);
unsafe impl<T> Send for SafetyHack<T> {}
unsafe impl<T> Sync for SafetyHack<T> {}

unsafe fn detach_guard<S: ThreadSafety>(guard: Guard<'_, S>) -> Guard<'static, S> {
	mem::transmute(guard)
}

#[allow(clippy::type_complexity)] // It's a mutable bounce pad, so the type is a bit long due to the through-access depth. Turning this pattern into a crate may be
struct ClearOnDrop<S: ThreadSafety>(Pin<Arc<Mutex<Option<Pin<Dereferenceable<Remember<S>>>>>>>);
impl<S: ThreadSafety> Drop for ClearOnDrop<S> {
	fn drop(&mut self) {
		*self.0.lock().unwrap() = None
	}
}
impl<S: ThreadSafety> Deref for ClearOnDrop<S> {
	type Target = Pin<Arc<Mutex<Option<Pin<Dereferenceable<Remember<S>>>>>>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
