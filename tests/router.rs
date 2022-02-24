use asteracea::components::Router;
use bumpalo::Bump;
use core::cell::Cell;
use lignin_html::render_fragment;
use rhizome::sync::Node;
use std::{any::TypeId, pin::Pin};

asteracea::component! {
	pub RouterTester()(
		path: &'bump str,
	) -> Sync

	with {
		let rest = Cell::default();
	} <*Router
		.path={path}
		.rest={&rest}

		//TODO: Using inline wildcards isn't ideal.
		->path={"/div/*"} <div !(rest.get())>
		->path={"/span/*"} <span !(rest.get())>
	/Router>
}

#[test]
fn div() {
	let root_node = Node::new(TypeId::of::<()>());
	let router =
		RouterTester::new(root_node.as_ref(), RouterTester::new_args_builder().build()).unwrap();
	let router = unsafe { Pin::new_unchecked(&router) };
	let bump = Bump::new();

	let vdom = router
		.render(
			&bump,
			RouterTester::render_args_builder()
				.path("/div/Hello!")
				.build(),
		)
		.unwrap();

	let mut html = String::new();
	render_fragment(&vdom, &mut html, 1000).unwrap();

	assert_eq!(&html, "<DIV>/Hello!</DIV>");
}

#[test]
fn span() {
	let root_node = Node::new(TypeId::of::<()>());
	let router =
		RouterTester::new(root_node.as_ref(), RouterTester::new_args_builder().build()).unwrap();
	let router = unsafe { Pin::new_unchecked(&router) };
	let bump = Bump::new();

	let vdom = router
		.render(
			&bump,
			RouterTester::render_args_builder()
				.path("/span/Router!")
				.build(),
		)
		.unwrap();

	let mut html = String::new();
	render_fragment(&vdom, &mut html, 1000).unwrap();

	assert_eq!(&html, "<SPAN>/Router!</SPAN>");
}
