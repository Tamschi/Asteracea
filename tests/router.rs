use asteracea::{__dependency_injection::ResourceNode, components::Router};
use bumpalo::Bump;
use core::cell::Cell;
use lignin_html::render_fragment;
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
		<div ^path={"/div/*"}
			!(rest.get())
		>
		<span ^path={"/span/*"}
			!(rest.get())
		>
	/Router>
}

#[test]
fn div() {
	let root_node = ResourceNode::new(TypeId::of::<()>());
	let router = RouterTester::new(&root_node, RouterTester::new_args_builder().build()).unwrap();
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
	let root_node = ResourceNode::new(TypeId::of::<()>());
	let router = RouterTester::new(&root_node, RouterTester::new_args_builder().build()).unwrap();
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
