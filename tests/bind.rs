use std::any::TypeId;

use bumpalo::Bump;
use rhizome::sync::Node;
use tap::Pipe;

asteracea::component! {
	Bound(
		priv _early: usize,
	)() []
}

asteracea::component! {
	Never

	#[allow(unreachable_code)]
	(
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
		spread if {false} bind move box <*BinderMover .late = {late}>
	]
}

asteracea::component! {
	Named()(
		late: usize = 1,
	) -> Sync

	bind priv bound: struct NamedBound <*Bound *_early = {late}>
}

asteracea::component! {
	NamedMoved()(
		late: usize = 1,
	) -> Sync

	bind priv bound: struct NamedMovedBound move <*Bound *_early = {late}>
}

#[test]
fn bind() {
	let root = Node::new(TypeId::of::<()>());
	let component = Binder::new(root.as_ref(), Binder::new_args_builder().build()).unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Binder::render_args_builder().build())
		.unwrap();

	BinderMover::new(root.as_ref(), BinderMover::new_args_builder().build())
		.unwrap()
		.pipe(Box::pin)
		.as_ref()
		.render(&bump, BinderMover::render_args_builder().build())
		.unwrap();

	Named::new(root.as_ref(), Named::new_args_builder().build())
		.unwrap()
		.pipe(Box::pin)
		.as_ref()
		.render(&bump, Named::render_args_builder().build())
		.unwrap();

	NamedMoved::new(root.as_ref(), NamedMoved::new_args_builder().build())
		.unwrap()
		.pipe(Box::pin)
		.as_ref()
		.render(&bump, NamedMoved::render_args_builder().build())
		.unwrap();
}
