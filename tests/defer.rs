// use bumpalo::Bump;
// use rhizome::Node;

// asteracea::component! {
// 	Deferred()() []
// }

// asteracea::component! {
// 	Deferrer()()

// 	[
// 		defer <*Deferred>
// 		spread if {false} defer box <*Deferrer>
// 	]
// }

// #[test]
// fn defer() {
// 	let root = Node::new_for::<()>();
// 	let component = Deferrer::new(&root.into(), Deferrer::new_args_builder().build());

// 	let bump = Bump::new();
// 	let _vdom = Box::pin(component)
// 		.as_ref()
// 		.render(&bump, Deferrer::render_args_builder().build());
// }
