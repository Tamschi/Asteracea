use std::any::TypeId;

use asteracea::__dependency_injection::ResourceNode;
use bumpalo::Bump;
use ergo_pin::ergo_pin;

asteracea::component! {
	Container()(..)

	<"custom-container"
		..
	>
}

asteracea::component! {
	Content()()

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
	let root = ResourceNode::new(TypeId::of::<()>());
	let parent = pin!(Parent::new(&root, Parent::new_args_builder().build()).unwrap());
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
