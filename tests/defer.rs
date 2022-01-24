use asteracea::__dependency_injection::ResourceNode;
use bumpalo::Bump;
use std::any::TypeId;

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
	let root = ResourceNode::new(TypeId::of::<()>());
	let component = Deferrer::new(&root, Deferrer::new_args_builder().build()).unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Deferrer::render_args_builder().build())
		.unwrap();
}
