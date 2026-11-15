pub mod r#type {
	#[doc(inline)]
	pub use crate::{__cell_type as cell, __pin_type as pin};
}

pub mod value {
	#[doc(inline)]
	pub use crate::{__cell_value as cell, __pin_value as pin};
}

pub mod access {
	#[doc(inline)]
	pub use crate::{__cell_access as cell, __pin_access as pin};
}
