use std::pin::Pin;

use debugless_unwrap::DebuglessUnwrap;
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
fn simple() {
	let root = Node::new_for::<()>();
	let component =
		Simple::new(&root.into(), Simple::new_args_builder().build()).debugless_unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Simple::render_args_builder().build());
}

asteracea::component! {
	Named()()

	box priv named <*Boxed priv boxed>
}

#[test]
fn named() {
	let root = Node::new_for::<()>();
	let component =
		Box::pin(Named::new(&root.into(), Named::new_args_builder().build()).debugless_unwrap());

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, Named::render_args_builder().build());

	let _: Boxed = component.named.boxed;
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
fn public() {
	use a_module::Public;

	let root = Node::new_for::<()>();
	let component =
		Box::pin(Public::new(&root.into(), Public::new_args_builder().build()).debugless_unwrap());

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, Public::render_args_builder().build());

	let _: a_module::Boxed = component.public.boxed;
}

asteracea::component! {
	Typed()()

	box priv named: struct TypedBoxed <*Boxed priv boxed>
}

#[test]
fn typed() {
	let root = Node::new_for::<()>();
	let component =
		Box::pin(Typed::new(&root.into(), Typed::new_args_builder().build()).debugless_unwrap());

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, Typed::render_args_builder().build());

	let typed: Pin<&TypedBoxed> = component.named.as_ref();
	let _: Boxed = typed.boxed;
}

struct BoxContainer {
	boxed: Boxed,
}

impl BoxContainer {
	fn boxed_pinned(self: Pin<&Self>) -> Pin<&Boxed> {
		unsafe {
			// SAFETY: Not moved out of.
			self.map_unchecked(|bc| &bc.boxed)
		}
	}
}

asteracea::component! {
	TypeReused()()

	box priv named: BoxContainer [
		with {
			#[allow(unused_variables)]
			let named = "This doesn't shadow the storage context for captures!";
		} <*Boxed priv boxed>
		<*{named.boxed_pinned()}>
	]
}

#[test]
fn reused() {
	let root = Node::new_for::<()>();
	let component = Box::pin(
		TypeReused::new(&root.into(), TypeReused::new_args_builder().build()).debugless_unwrap(),
	);

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, TypeReused::render_args_builder().build());

	let typed: &Pin<Box<BoxContainer>> = &component.named;
	let _: Boxed = typed.boxed;
}

asteracea::component! {
	VisIgnored()()

	box priv b: BoxContainer
		// There's no good way to check the visibility here (since the declaration isn't emitted),
		// so it's possible to use a mismatching one for fields on externally-defined storage types.
		<*Boxed pub boxed>
}

asteracea::component! {
	Multi()()

	[
		box <*Boxed priv boxed>
		box <*Boxed priv boxed>
	]
}

#[test]
fn multi() {
	let root = Node::new_for::<()>();
	let component = Multi::new(&root.into(), Multi::new_args_builder().build()).debugless_unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Multi::render_args_builder().build());
}

asteracea::component! {
	Nested()()

	[
		box [
			box <*Boxed>
			box <*Boxed>
		]
		box box <*Boxed>
	]
}
