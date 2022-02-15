use crate::{include::render_callback::RenderOnce, __::Built};
use bumpalo::Bump;
use lignin::{Guard, ThreadSafety};
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
	pub Relaxed()
	<S: ThreadSafety>(
		unless?: impl FnOnce() -> bool,
		__Asteracea__anonymous_content: (NoParentParameters, Box<RenderOnce<'_, 'bump, S>>),
	) -> Guard<'bump, S>

	{
		//TODO
		__Asteracea__anonymous_content.1(bump)?
	}
}
