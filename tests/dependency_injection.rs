#![cfg(FALSE)] //TODO

use bumpalo::Bump;
use rhizome::{Node, TypeKey};
use std::{any::Any, convert::Infallible};
use tap::Pipe;

struct InjectedStruct;

impl TypeKey<Infallible> for InjectedStruct {
	fn provision() -> rhizome::Provision<Infallible, Box<dyn Any>> {
		rhizome::Provision::at_root(|_node| Ok(Box::new(InjectedStruct)))
	}
}

trait InjectedTrait: Any {}
impl TypeKey<Infallible> for dyn InjectedTrait {
	fn provision() -> rhizome::Provision<Infallible, Box<dyn Any>> {
		rhizome::Provision::at_root(|_node| {
			impl InjectedTrait for InjectedStruct {}
			let trait_wise: Box<dyn InjectedTrait> = Box::new(InjectedStruct);
			Ok(Box::new(trait_wise))
		})
	}
}

struct Unavailable;
impl TypeKey<Infallible> for Unavailable {}

//

asteracea::component! {
	Injections(
		ref a: InjectedStruct,
		priv ref b: InjectedStruct,
		ref c: dyn InjectedTrait,
		priv ref d: dyn InjectedTrait,
	)()

	new with {
		assert!(a.is_some());
		assert!(b.is_some());
		assert!(c.is_some());
		assert!(d.is_some());

		let b = ();
		let d = ();
	}

	with {
		assert!(self.b.is_some());
		assert!(self.d.is_some());
	} []
}

asteracea::component! {
	OptionalInjections(
		ref a?: InjectedStruct,
		priv ref b?: InjectedStruct,
		ref c?: dyn InjectedTrait,
		priv ref d?: dyn InjectedTrait,
		ref e?: Unavailable,
		priv ref f?: Unavailable,
	)()

	new with {
		assert!(a.is_some());
		assert!(self.b.is_some());
		assert!(c.is_some());
		assert!(self.d.is_some());
		assert!(e == None);
		assert!(self.f == None);

		let b = ();
		let d = ();
	}

	with {
		assert!(self.b.is_some());
		assert!(self.d.is_some());
		assert!(self.f == None);
	} []
}

#[test]
fn dependency_injection() {
	let root = std::sync::Arc::new(Node::new_for::<()>());
	let bump = Bump::new();

	Injections::new(&root, Injections::new_args_builder().build())
		.unwrap()
		.pipe(Box::pin)
		.as_ref()
		.render(&bump, Injections::render_args_builder().build())
		.unwrap();
}
