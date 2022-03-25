use asteracea::services::Invalidator;
use bumpalo::Bump;
use debugless_unwrap::DebuglessUnwrap;
use ergo_pin::ergo_pin;
use rhizome::sync::{Inject, Node};
use std::{any::TypeId, task::Context};
use this_is_fine::FineExt;

asteracea::component! {
	Container()(..)

	new with {
		<dyn Invalidator>::inject(
			local_resource_node.borrow(),
			|_: Option<&mut Context<'_>>| unimplemented!(),
		).not_fine().debugless_unwrap();
	}

	<"custom-container"
		..
	>
}

asteracea::component! {
	Content(
		dyn _invalidator: dyn Invalidator,
	)()

	<"custom-content">
}

asteracea::component! {
	Parent()() -> Sync

	<*Container
		<*Content>
	>
}

#[test]
#[ergo_pin]
fn content_in_container() {
	let root = Node::new(TypeId::of::<()>());
	let parent = pin!(
		Parent::new(root.as_ref(), Parent::new_args_builder().build())
			.unwrap()
			.0
	);
	let bump = Bump::new();
	let vdom = parent
		.as_ref()
		.render(&bump, Parent::render_args_builder().build())
		.unwrap();
	let mut html = String::new();
	lignin_html::render_fragment(&vdom, &mut html, 3).unwrap();
	assert_eq!(
		html,
		"<custom-container><custom-content></custom-content></custom-container>"
	)
}
