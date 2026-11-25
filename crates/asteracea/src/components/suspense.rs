use crate::{
	include::{
		async_::{AsyncContent, ContentSubscription, Synchronized},
		render_callback::RenderOnce,
	},
	services::{ContentRuntime, Invalidator},
	__::Built,
};
use bon::Builder;
use lignin::{Node, ThreadSafety};
use std::cell::UnsafeCell;

#[derive(Builder)]
pub struct NoParentParameters {}
impl Built for NoParentParameters {
	type Builder = NoParentParametersBuilder<no_parent_parameters_builder::Empty>;

	fn builder() -> Self::Builder {
		Self::builder()
	}
}

asteracea::components! {
	/// Renders `'spinner` unless `'ready` has finished construction.
	///
	/// `'ready`'s construction is scheduled automatically.
	pub Suspense(
		pub(self) dyn runtime: dyn ContentRuntime,
	){
		'spinner,
		async 'ready,
	} -> web {
		#[attribute]
		let self.subscription = UnsafeCell::<Option<ContentSubscription>>::new(None);

		[{
			match ready.1.synchronize(unsafe{&mut *self.subscription.get()}) {
				Synchronized::Unchanged => (),
				Synchronized::Reset(future) => self.runtime.start_content_future(future, self.invalidator.clone()),
			}

			ready.1.render(bump).unwrap_or_else(|| (spinner.1)(bump))?;
		}]
	}
}
