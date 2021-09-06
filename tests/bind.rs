#![allow(unreachable_code)]

use bumpalo::Bump;
use rhizome::Node;

asteracea::component! {
	Bound(
		priv _early: usize,
	)() []
}

asteracea::component! {
	Never(
		priv _early: usize,
	)()

	new with { unreachable!(); }

	[]
}

asteracea::component! {
	Binder()(
		late: usize = 1,
	) -> Sync

	[
		bind <*Bound *_early = {late}>
		spread if {false} bind <*Never *_early = {late}>
		spread if {false} bind box <*Binder .late = {late}>
	]
}

asteracea::component! {
	BinderMover()(
		late: usize = 1,
	) -> Sync

	[
		bind move <*Bound *_early = {late}>
		spread if {false} bind move <*Never *_early = {late}>
		spread if {false} bind move box <*Binder .late = {late}>
	]
}

asteracea::component! {
	Named()(
		late: usize = 1,
	) -> Sync

	[
		bind priv bound: struct NamedBound <*Bound *_early = {late}>
	]
}

asteracea::component! {
	NamedMoved()(
		late: usize = 1,
	) -> Sync

	[
		bind priv bound: struct NamedMovedBound move <*Bound *_early = {late}>
	]
}

#[test]
fn bind() {
	let root = Node::new_for::<()>();
	let component = Binder::new(&root.into(), Binder::new_args_builder().build()).unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Binder::render_args_builder().build())
		.unwrap();
}
