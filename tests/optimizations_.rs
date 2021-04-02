use std::mem::size_of;

use typed_builder::TypedBuilder;

fn stateless_components_are_zero_sized() {
	#[derive(TypedBuilder)]
	#[builder(doc)]
	struct EmptyNewArgs<'NEW, 'a: 'NEW> {
		#[builder(default, setter(skip))]
		__Asteracea__phantom: ::std::marker::PhantomData<(&'NEW (), &'a ())>,
	}
	#[derive(TypedBuilder)]
	#[builder(doc)]
	struct EmptyRenderArgs<'RENDER, 'a, 'bump: 'RENDER> {
		#[builder(default, setter(skip))]
		__Asteracea__phantom: ::std::marker::PhantomData<(&'RENDER (), &'a (), &'bump ())>,
	}
	struct Empty {}
	impl Empty {}
	impl Empty {
		pub fn new<'a>(
			parent_node: &::std::sync::Arc<asteracea::rhizome::Node>,
			EmptyNewArgs {
				__Asteracea__phantom: _,
			}: EmptyNewArgs<'_, 'a>,
		) -> ::std::result::Result<Self, ::asteracea::error::Escalation>
		where
			Self: 'a + 'static,
		{
			let node =
				asteracea::rhizome::extensions::TypeTaggedNodeArc::derive_for::<Self>(parent_node);
			let mut node = node;
			{}
			{}
			let node = node.into_arc();
			::std::result::Result::Ok(Empty {})
		}
		pub fn new_args_builder<'NEW, 'a: 'NEW>() -> EmptyNewArgsBuilder<'NEW, 'a, ()> {
			EmptyNewArgs::builder()
		}
		pub fn render<'a, 'bump>(
			self: ::std::pin::Pin<&'a Self>,
			bump: &'bump asteracea::bumpalo::Bump,
			EmptyRenderArgs {
				__Asteracea__phantom: _,
			}: EmptyRenderArgs<'_, 'a, 'bump>,
		) -> ::std::result::Result<
			impl Empty__Asteracea__AutoSafe<
				::asteracea::lignin::Node<'bump, ::asteracea::lignin::ThreadBound>,
			>,
			::asteracea::error::Escalation,
		> {
			let this = self;
			::std::result::Result::Ok(
				::asteracea::lignin::Node::<'bump, _>::Text {
					text: "",
					dom_binding: None,
				}
				.prefer_thread_safe(),
			)
		}
		pub fn render_args_builder<'RENDER, 'a, 'bump: 'RENDER>(
		) -> EmptyRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
			EmptyRenderArgs::builder()
		}
		#[doc(hidden)]
		pub fn __Asteracea__ref_render_args_builder<'RENDER, 'a, 'bump: 'RENDER>(
			&self,
		) -> EmptyRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
			let _ = self;
			EmptyRenderArgs::builder()
		}
	}
	/// An alias for [`$crate::auto_safety::AutoSafe`] with custom visibility.
	trait Empty__Asteracea__AutoSafe<BoundVariant>:
		::lignin::auto_safety::AutoSafe<BoundVariant>
	where
		BoundVariant: ::lignin::Vdom<ThreadSafety = ::lignin::ThreadBound>,
	{
	}
	impl<T, BoundVariant> Empty__Asteracea__AutoSafe<BoundVariant> for T
	where
		T: ::lignin::auto_safety::AutoSafe<BoundVariant>,
		BoundVariant: ::lignin::Vdom<ThreadSafety = ::lignin::ThreadBound>,
	{
	}
	{
		assert_eq!(size_of::<Empty>(), 0)
	}
}
