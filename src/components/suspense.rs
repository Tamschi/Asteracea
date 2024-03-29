use crate::{
	include::{
		async_::{AsyncContent, ContentSubscription, Synchronized},
		render_callback::RenderOnce,
	},
	services::{ContentRuntime, Invalidator},
	__::Built,
};
use lignin::{Node, ThreadSafety};
use std::cell::UnsafeCell;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct NoParentParameters {}
impl Built for NoParentParameters {
	type Builder = NoParentParametersBuilder<()>;

	fn builder() -> Self::Builder {
		Self::builder()
	}
}

asteracea::component! {
	/// Renders `'spinner` unless `'ready` has finished construction.
	///
	/// `'ready`'s construction is scheduled automatically.
	pub Suspense(
		priv dyn runtime: dyn ContentRuntime,
		priv dyn invalidator?: dyn Invalidator,
	)<S: 'bump + ThreadSafety>(
		spinner: (NoParentParameters, Box<RenderOnce<'_, 'bump, S>>),
		mut ready: (NoParentParameters, AsyncContent<'_, RenderOnce<'_, 'bump, S>>),
	) -> Node::<'bump, S>

	let self.subscription = UnsafeCell::<Option<ContentSubscription>>::new(None);

	{
		match ready.1.synchronize(unsafe{&mut *self.subscription.get()}) {
			Synchronized::Unchanged => (),
			Synchronized::Reset(future) => self.runtime.start_content_future(future, self.invalidator.clone()),
		}

		ready.1.render(bump).unwrap_or_else(|| (spinner.1)(bump))?
	}
}
