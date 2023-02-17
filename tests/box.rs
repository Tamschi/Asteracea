use asteracea::substrates::web as substrate;
use bumpalo::Bump;
use debugless_unwrap::DebuglessUnwrap;
use rhizome::sync::Node;
use std::{any::TypeId, pin::Pin};

asteracea::component! { substrate =>
	Boxed()() []
}

// asteracea::component! { substrate =>
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

asteracea::component! { substrate =>
	Simple()()

	box <*Boxed>
}

#[test]
fn simple() {
	let root = Node::new(TypeId::of::<()>());
	let component =
		Simple::new(root.as_ref(), Simple::new_args_builder().build()).debugless_unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Simple::render_args_builder().build());
}

asteracea::component! { substrate =>
	Named()()

	box priv named <*Boxed priv boxed>
}

#[test]
fn named() {
	let root = Node::new(TypeId::of::<()>());
	let component =
		Box::pin(Named::new(root.as_ref(), Named::new_args_builder().build()).debugless_unwrap());

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, Named::render_args_builder().build());

	let _: Boxed = component.named.boxed;
}

mod a_module {
	use asteracea::substrates::web as substrate;

	asteracea::component! { substrate =>
		pub Boxed()() []
	}

	asteracea::component! { substrate =>
		pub Public()()

		box pub public <*Boxed pub boxed>
	}
}

#[test]
fn public() {
	use a_module::Public;

	let root = Node::new(TypeId::of::<()>());
	let component =
		Box::pin(Public::new(root.as_ref(), Public::new_args_builder().build()).debugless_unwrap());

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, Public::render_args_builder().build());

	let _: a_module::Boxed = component.public.boxed;
}

asteracea::component! { substrate =>
	Typed()()

	box priv named: struct TypedBoxed <*Boxed priv boxed>
}

#[test]
fn typed() {
	let root = Node::new(TypeId::of::<()>());
	let component =
		Box::pin(Typed::new(root.as_ref(), Typed::new_args_builder().build()).debugless_unwrap());

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

asteracea::component! { substrate =>
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
	let root = Node::new(TypeId::of::<()>());
	let component = Box::pin(
		TypeReused::new(root.as_ref(), TypeReused::new_args_builder().build()).debugless_unwrap(),
	);

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, TypeReused::render_args_builder().build());

	let typed: &Pin<Box<BoxContainer>> = &component.named;
	let _: Boxed = typed.boxed;
}

asteracea::component! { substrate =>
	pub VisIgnored()()

	box priv b: BoxContainer
		// There's no good way to check the visibility here (since the declaration isn't emitted),
		// so it's possible to use a mismatching one for fields on externally-defined storage types.
		<*Boxed pub boxed>
}

asteracea::component! { substrate =>
	Multi()()

	[
		box <*Boxed priv boxed>
		box <*Boxed priv boxed>
	]
}

#[test]
fn multi() {
	let root = Node::new(TypeId::of::<()>());
	let component = Multi::new(root.as_ref(), Multi::new_args_builder().build()).debugless_unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Multi::render_args_builder().build());
}

asteracea::component! { substrate =>
	pub Nested()()

	[
		box [
			box <*Boxed>
			box <*Boxed>
		]
		box box <*Boxed>
	]
}
