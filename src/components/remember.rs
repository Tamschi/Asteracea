use crate::{
	include::{__for_::BoxedAnyOneK, private::Dereferenceable, render_callback::RenderOnce},
	services::Invalidator,
	__::Built,
};
use bumpalo::Bump;
use lignin::{Guard, ThreadSafety};
use rhizome::sync::Inject;
use std::{
	any::Any,
	collections::hash_map::DefaultHasher,
	hash::Hasher,
	marker::PhantomPinned,
	pin::Pin,
	ptr::NonNull,
	sync::{
		atomic::{AtomicBool, AtomicU32, Ordering},
		Arc, Mutex,
	},
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
	pub Remember(
		dyn invalidator?: dyn Invalidator,
	)
	<S: ThreadSafety>(
		// for_unchanged?: &R,
		or_unless?: impl FnOnce() -> bool,
		__Asteracea__anonymous_content: (NoParentParameters, Box<RenderOnce<'_, 'bump, S>>),
	) -> Guard<'bump, S>

	new with {
		let this = Arc::pin(Mutex::new(None::<Pin<Dereferenceable<Self>>>));
		if let Some(invalidator) = invalidator {
			<dyn Invalidator>::inject(node.as_ref(), {
				let this = Pin::clone(&this);
				move || {
					if let Some(this) = this.lock().unwrap().as_ref() {
						this.invalidated.store(true, Ordering::Release)
					}
					invalidator.invalidate()
				}
			}).1.ok().expect("Failed to inject new invalidator in `Remember`.");
		}
	}
	let self._pinned = PhantomPinned::default();
	let self.this_set: AtomicBool = false.into();
	let self.this: Pin<Arc<Mutex<Option<Pin<Dereferenceable<Self>>>>>> = this;

	let self.invalidated: AtomicBool = true.into();

	// let self.when_unchanged_stored: Mutex<Option<Box<dyn Send + Any>>> = None.into();

	let self.hasher: Mutex<DefaultHasher> = {
		let mut hasher = DefaultHasher::new();
		hasher.write_u32(GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed));
		hasher.into()
	};

	{
		if !self.this_set.load(Ordering::Acquire) {
			*self.this.lock().unwrap() = Some(unsafe {
				Pin::new_unchecked(Dereferenceable::new(self.get_ref().into()))
			});
			self.this_set.store(true, Ordering::Release);
		}

		if self.invalidated.swap(false, Ordering::SeqCst)
			|| or_unless.map(|f| f()).unwrap_or(false) {
				//TODO
		}

		//TODO
		__Asteracea__anonymous_content.1(bump)?
	}
}
