pub mod r#type {
	#[doc(inline)]
	pub use crate::{
		__cell_cyclic_type as cell_cyclic, __cell_type as cell, __computed_type as computed,
		__computed_uncached_mut_type as computed_uncached_mut,
		__computed_uncached_type as computed_uncached, __distinct_type as distinct,
		__pin_type as pin,
	};
}

pub mod value {
	#[doc(inline)]
	pub use crate::{
		__cell_cyclic_value as cell_cyclic, __cell_value as cell,
		__computed_uncached_mut_value as computed_uncached_mut,
		__computed_uncached_value as computed_uncached, __computed_value as computed,
		__distinct_value as distinct, __pin_value as pin,
	};
}

pub mod access {
	#[doc(inline)]
	pub use crate::{
		__cell_access as cell, __cell_cyclic_access as cell_cyclic, __computed_access as computed,
		__computed_uncached_access as computed_uncached,
		__computed_uncached_mut_access as computed_uncached_mut, __distinct_access as distinct,
		__pin_access as pin,
	};
}
