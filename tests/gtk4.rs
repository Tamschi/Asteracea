use asteracea::substrates::gtk4;
use bumpalo::Bump;
use ergo_pin::ergo_pin;
use rhizome::sync::Node;
use std::any::TypeId;

asteracea::component! { gtk4 =>
	pub Empty()() []
}

asteracea::component! { gtk4 =>
	pub ApplicationWindow()()

	<ApplicationWindow .application={}>
}

#[test]
#[ergo_pin]
fn empty() {
	let root = Node::new(TypeId::of::<()>());
	let component = pin!(Empty::new(root.as_ref(), Empty::new_args_builder().build()).unwrap());
	let bump = Bump::new();
	let vdom = component
		.as_ref()
		.render(&bump, Empty::render_args_builder().build())
		.unwrap();

	dbg!(vdom);
}

#[test]
#[ergo_pin]
fn application_window() {
	let root = Node::new(TypeId::of::<()>());
	let component = pin!(ApplicationWindow::new(
		root.as_ref(),
		ApplicationWindow::new_args_builder().build()
	)
	.unwrap());
	let bump = Bump::new();
	let vdom = component
		.as_ref()
		.render(&bump, ApplicationWindow::render_args_builder().build())
		.unwrap();

	dbg!(vdom);
}
