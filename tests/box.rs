use asteracea::error::ExtractableResolutionError;
use lignin::bumpalo::Bump;
use rhizome::Node;

asteracea::component! {
	pub Boxed()() []
}

// asteracea::component! {
// 	Countdown()(
// 		i: usize,
// 	)

// 	[
// 		!{i}
// 		spread if {i > 0} [
// 			"\n"
// 			defer box <*Countdown .i = {i - 1}>
// 		]
// 	]
// }

asteracea::component! {
	Simple()()

	box <*Boxed>
}

#[test]
fn simple() -> Result<(), ExtractableResolutionError> {
	let root = Node::new_for::<()>();
	let component = Simple::new(&root.into(), Simple::new_args_builder().build())?;

	let bump = Bump::new();
	let _vdom = component.render(&bump, Simple::render_args_builder().build());

	Ok(())
}

asteracea::component! {
	Named()()

	box priv named <*Boxed priv boxed>
}

#[test]
fn named() -> Result<(), ExtractableResolutionError> {
	let root = Node::new_for::<()>();
	let component = Named::new(&root.into(), Named::new_args_builder().build())?;

	let bump = Bump::new();
	let _vdom = component.render(&bump, Named::render_args_builder().build());

	let _: Boxed = (*component.named).boxed;

	Ok(())
}

mod a_module {
	asteracea::component! {
		pub Boxed()() []
	}

	asteracea::component! {
		pub Public()()

		box pub public <*Boxed pub boxed>
	}
}

#[test]
fn public() -> Result<(), ExtractableResolutionError> {
	use a_module::Public;

	let root = Node::new_for::<()>();
	let component = Public::new(&root.into(), Public::new_args_builder().build())?;

	let bump = Bump::new();
	let _vdom = component.render(&bump, Public::render_args_builder().build());

	let _: Boxed = (*component.public).boxed;

	Ok(())
}

asteracea::component! {
	Typed()()

	box priv named: struct TypedBoxed <*Boxed priv boxed>
}

#[test]
fn typed() -> Result<(), ExtractableResolutionError> {
	let root = Node::new_for::<()>();
	let component = Typed::new(&root.into(), Typed::new_args_builder().build())?;

	let bump = Bump::new();
	let _vdom = component.render(&bump, Typed::render_args_builder().build());

	let typed: TypedBoxed = *component.named;
	let _: Boxed = typed.boxed;

	Ok(())
}

asteracea::component! {
	TypeReused()()

	box priv named: TypedBoxed <*Boxed priv boxed>
}

#[test]
fn reused() -> Result<(), ExtractableResolutionError> {
	let root = Node::new_for::<()>();
	let component = TypeReused::new(&root.into(), TypeReused::new_args_builder().build())?;

	let bump = Bump::new();
	let _vdom = component.render(&bump, TypeReused::render_args_builder().build());

	let typed: TypedBoxed = *component.named;
	let _: Boxed = typed.boxed;

	Ok(())
}

asteracea::component! {
	Multi()()

	[
		box <*Boxed priv boxed>
		box <*Boxed priv boxed>
	]
}

#[test]
fn multi() -> Result<(), ExtractableResolutionError> {
	let root = Node::new_for::<()>();
	let component = Multi::new(&root.into(), Multi::new_args_builder().build())?;

	let bump = Bump::new();
	let _vdom = component.render(&bump, Multi::render_args_builder().build());

	Ok(())
}
