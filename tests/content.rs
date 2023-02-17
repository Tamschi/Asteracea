use std::any::TypeId;

use bumpalo::Bump;
use ergo_pin::ergo_pin;
use rhizome::sync::Node;

asteracea::component! { substrate =>
	Container()(..)

	<"custom-container"
		..
	>
}

asteracea::component! { substrate =>
	Content()()

	<"custom-content">
}

asteracea::component! { substrate =>
	Parent()() -> Sync

	<*Container
		<*Content>
	>
}

#[test]
#[ergo_pin]
fn content_in_container() {
	let root = Node::new(TypeId::of::<()>());
	let parent = pin!(Parent::new(root.as_ref(), Parent::new_args_builder().build()).unwrap());
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
