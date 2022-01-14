use bumpalo::Bump;
use rhizome::Node;

asteracea::component! {
	Deferred()() []
}

asteracea::component! {
	Never
	#[allow(unreachable_code)]
	()()

	new with { unreachable!(); }

	[]
}

asteracea::component! {
	Deferrer()() -> Sync

	[
		defer <*Deferred>
		spread if {false} defer <*Never>
		spread if {false} defer box <*Deferrer>
	]
}

asteracea::component! {
	Named()() -> Sync

	defer priv deferred: struct NamedDeferred <*Deferred>
}

#[test]
fn defer() {
	let root = Node::new_for::<()>();
	let component = Deferrer::new(&root.into(), Deferrer::new_args_builder().build()).unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Deferrer::render_args_builder().build())
		.unwrap();
}
