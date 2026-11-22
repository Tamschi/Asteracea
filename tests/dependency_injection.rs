mod _sub;

use std::{any::TypeId, pin::Pin, sync::Arc};

use asteracea::{
	components,
	substrate::bounds::{Projects, Raises},
};
use rhizome::{Injectable, Repository};

impl Injectable for dyn Trait {
	const PROVISION_AT: &'static [TypeId] = &[];

	type Provided = Pin<Arc<Self>>;

	type Error = ();

	fn provision(
		_at: &mut rhizome::Waypoint<'_, impl ?Sized + Repository>,
		_matched_tag: Option<TypeId>,
	) -> Result<Pin<Arc<Self>>, Self::Error> {
		Err(())
	}

	fn provide(self: &Pin<Arc<Self>>) -> Self::Provided {
		Pin::clone(self)
	}
}

impl Injectable for Struct {
	const PROVISION_AT: &'static [TypeId] = &[];

	type Provided = Struct;

	type Error = !;

	fn provision(
		_at: &mut rhizome::Waypoint<'_, impl ?Sized + Repository>,
		_matched_tag: Option<TypeId>,
	) -> Result<Pin<Arc<Self>>, Self::Error> {
		Ok(Arc::pin(Struct { a: (), b: () }))
	}

	fn provide(self: &Pin<Arc<Self>>) -> Self::Provided {
		(**self).clone()
	}
}

trait Trait: Send + Sync {}

#[derive(Clone)]
pub struct Struct {
	a: (),
	b: (),
}

components! {
	#![substrate = S where S: 'static + Raises<()> + Raises<!> + Projects<()>]

	pub Test1(
		dyn t: dyn Trait,
		[dyn] Struct { a, b }: Struct,
		{dyn} t2: dyn Trait,
	) => {
		({ let _ = (t, t2); })
		[{ let _ = (a, b, t2); }]
	}

	pub Test2(
		s: Struct,
		dyn t?: dyn Trait,
		//TODO: Ensure `dyn` has proper semantic highlights here.
		[dyn] Struct { a, b }?: Struct = s,
		{dyn} t2?: dyn Trait,
	) => {
		({ let _ = (t, t2); })
		[{ let _ = (a, b, t2); }]
	}
}
