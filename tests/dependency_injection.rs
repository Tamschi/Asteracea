mod _sub;

use std::{any::TypeId, pin::Pin, sync::Arc};

use asteracea::{components, substrate::bounds::Raises};
use rhizome::Injectable;

impl Injectable for dyn Trait {
	const PROVISION_AT: &'static [TypeId] = &[];

	type Provided = Pin<Arc<Self>>;

	type Error = ();

	fn provision(
		_at: &mut rhizome::Waypoint<'_>,
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
		_at: &mut rhizome::Waypoint<'_>,
		_matched_tag: Option<TypeId>,
	) -> Result<Pin<Arc<Self>>, Self::Error> {
		Ok(Arc::pin(Struct { a: (), b: () }))
	}

	fn provide(self: &Pin<Arc<Self>>) -> Self::Provided {
		**self
	}
}

trait Trait: Send + Sync {}

#[derive(Clone, Copy)]
struct Struct {
	a: (),
	b: (),
}

components! {
	#![substrate = S where S: Raises<()> + Raises<!>]

	pub Test(
		_u: usize,
		(dyn) _t: dyn Trait,
		(dyn) Struct { a, b }: Struct,
	) => {

	}
}
