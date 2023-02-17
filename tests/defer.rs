use std::any::TypeId;

use bumpalo::Bump;
use rhizome::sync::Node;

asteracea::component! { substrate =>
	Deferred()() []
}

asteracea::component! { substrate =>
	Never
	#[allow(unreachable_code)]
	()()

	new with { unreachable!(); }

	[]
}

asteracea::component! { substrate =>
	Deferrer()() -> Sync

	[
		defer <*Deferred>
		spread if {false} defer <*Never>
		spread if {false} defer box <*Deferrer>
	]
}

asteracea::component! { substrate =>
	Named()() -> Sync

	defer priv deferred: struct NamedDeferred <*Deferred>
}

#[test]
fn defer() {
	let root = Node::new(TypeId::of::<()>());
	let component = Deferrer::new(root.as_ref(), Deferrer::new_args_builder().build()).unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Deferrer::render_args_builder().build())
		.unwrap();
}
