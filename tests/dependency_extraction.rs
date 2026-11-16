mod _sub;

use asteracea::{components, substrate::bounds::IsSubstrate};
use rhizome::derive_dependency_sync;

trait Injectable {}

derive_dependency_sync!(dyn Injectable);

components! {
	#![substrate = S where S: IsSubstrate]

	pub Test(
		a: usize,
		dyn b: dyn Injectable,
	) => {

	}
}
