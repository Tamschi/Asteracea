use bumpalo::Bump;
use rhizome::sync::Node;
use std::any::TypeId;

asteracea::component! {
	Let()()

	let a = 1;
	[
		!(a)
		// let a = 2; //TODO
		!(a)
		!(a)
	]
}

//TODO: Ensure this renders accurately.

#[test]
fn r#let() {
	let root = Node::new(TypeId::of::<()>());
	let component = Let::new(root.as_ref(), Let::new_args_builder().build()).unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Let::render_args_builder().build())
		.unwrap();
}
