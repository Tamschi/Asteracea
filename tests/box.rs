mod old {
	#![allow(deprecated)]

	use std::{any::TypeId, pin::Pin};

	use bumpalo::Bump;

	use debugless_unwrap::DebuglessUnwrapExt;

	use rhizome::sync::Node;

	asteracea::component! {
		Boxed()() []
	}

	asteracea::component! {
		Simple()()

		box <*Boxed>
	}

	#[test]
	pub(crate) fn simple() {
		let root = Node::new(TypeId::of::<()>());
		let component =
			Simple::new(root.as_ref(), Simple::new_args_builder().build()).debugless_unwrap();

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
	pub(crate) fn named() {
		let root = Node::new(TypeId::of::<()>());
		let component = Box::pin(
			Named::new(root.as_ref(), Named::new_args_builder().build()).debugless_unwrap(),
		);

		let bump = Bump::new();
		let _vdom = component
			.as_ref()
			.render(&bump, Named::render_args_builder().build());

		let _: Boxed = component.named.boxed;
	}

	pub(crate) mod a_module {
		asteracea::component! {
			pub Boxed()() -> Sync []
		}

		asteracea::component! {
			pub Public()() -> Sync

			box pub public <*Boxed pub boxed>
		}
	}

	#[test]
	pub(crate) fn public() {
		use a_module::Public;

		let root = Node::new(TypeId::of::<()>());
		let component = Box::pin(
			Public::new(root.as_ref(), Public::new_args_builder().build()).debugless_unwrap(),
		);

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
	pub(crate) fn typed() {
		let root = Node::new(TypeId::of::<()>());
		let component = Box::pin(
			Typed::new(root.as_ref(), Typed::new_args_builder().build()).debugless_unwrap(),
		);

		let bump = Bump::new();
		let _vdom = component
			.as_ref()
			.render(&bump, Typed::render_args_builder().build());

		let typed: Pin<&TypedBoxed> = component.named.as_ref();
		let _: Boxed = typed.boxed;
	}

	pub(crate) struct BoxContainer {
		pub(crate) boxed: Boxed,
	}

	impl BoxContainer {
		pub(crate) fn boxed_pinned(self: Pin<&Self>) -> Pin<&Boxed> {
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
	pub(crate) fn reused() {
		let root = Node::new(TypeId::of::<()>());
		let component = Box::pin(
			TypeReused::new(root.as_ref(), TypeReused::new_args_builder().build())
				.debugless_unwrap(),
		);

		let bump = Bump::new();
		let _vdom = component
			.as_ref()
			.render(&bump, TypeReused::render_args_builder().build());

		let typed: &Pin<Box<BoxContainer>> = &component.named;
		let _: Boxed = typed.boxed;
	}

	asteracea::component! {
		pub VisIgnored()() -> Sync

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
	pub(crate) fn multi() {
		let root = Node::new(TypeId::of::<()>());
		let component =
			Multi::new(root.as_ref(), Multi::new_args_builder().build()).debugless_unwrap();

		let bump = Bump::new();
		let _vdom = Box::pin(component)
			.as_ref()
			.render(&bump, Multi::render_args_builder().build());
	}

	asteracea::component! {
		pub Nested()() -> Sync

		[
			box [
				box <*Boxed>
				box <*Boxed>
			]
			box box <*Boxed>
		]
	}
}

use std::{any::TypeId, pin::Pin};

use bumpalo::Bump;

use debugless_unwrap::DebuglessUnwrapExt;

use rhizome::sync::Node;

asteracea::components! {
	Boxed -> web {}
}

asteracea::components! {
	Simple -> web {
		box Boxed;
	}

}

#[test]
pub(crate) fn simple() {
	let root = Node::new(TypeId::of::<()>());
	let component =
		Simple::new(root.as_ref(), Simple::new_args_builder().build()).debugless_unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Simple::render_args_builder().build());
}

asteracea::components! {
	Named -> web {
		box as self.named { Boxed as self.boxed; }
	}
}

#[test]
pub(crate) fn named() {
	let root = Node::new(TypeId::of::<()>());
	let component =
		Box::pin(Named::new(root.as_ref(), Named::new_args_builder().build()).debugless_unwrap());

	let bump = Bump::new();
	let _vdom = component
		.as_ref()
		.render(&bump, Named::render_args_builder().build());

	let _: Boxed = component.named.boxed;
}

pub(crate) mod a_module {
	asteracea::components! {
		pub Boxed -> web {}

		pub Public -> web {
			box as pub self.public { Boxed as pub self.boxed; }
		}
	}
}

#[test]
pub(crate) fn public() {
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

asteracea::components! {
	Typed -> web {
		box as self.named: struct TypedBoxed { Boxed as self.boxed; }
	}
}

#[test]
pub(crate) fn typed() {
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

pub(crate) struct BoxContainer {
	pub(crate) boxed: Boxed,
}

impl BoxContainer {
	pub(crate) fn boxed_pinned(self: Pin<&Self>) -> Pin<&Boxed> {
		unsafe {
			// SAFETY: Not moved out of.
			self.map_unchecked(|bc| &bc.boxed)
		}
	}
}

asteracea::components! {
	TypeReused -> web {
		box as self.named: BoxContainer {
			#[allow(unused_variables)]
			let named = "This doesn't shadow the storage context for captures!";
			Boxed as self.boxed;
			*{self.named.boxed_pinned()}
		}
	}
}

#[test]
pub(crate) fn reused() {
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

asteracea::components! {
	pub VisIgnored -> web {
		box as self.b: BoxContainer {
			// There's no good way to check the visibility here (since the declaration isn't emitted),
			// so it's possible to use a mismatching one for fields on externally-defined storage types.
			Boxed as pub self.boxed;
		}
	}

	Multi -> web {
		box Boxed as self.boxed;
		box Boxed as self.boxed;
	}
}

#[test]
pub(crate) fn multi() {
	let root = Node::new(TypeId::of::<()>());
	let component = Multi::new(root.as_ref(), Multi::new_args_builder().build()).debugless_unwrap();

	let bump = Bump::new();
	let _vdom = Box::pin(component)
		.as_ref()
		.render(&bump, Multi::render_args_builder().build());
}

asteracea::components! {
	pub Nested -> web {
		box {
			box Boxed;
			box Boxed;
		}
		box box Boxed;
	}
}
