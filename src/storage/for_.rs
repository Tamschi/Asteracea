use std::{borrow::Borrow, pin::Pin};

use linotype::{Linotype, PinningLinotype};

use crate::error::Escalation;

/// Storage for [`for`](`For`) expressions.
pub struct For<'a, Storage, K = MixedKey> {
	linotype: Pin<Linotype<K, Storage>>,
	factory: Box<dyn 'a + FnMut() -> Result<Storage, Escalation>>,
}

impl<'a, Storage, K> For<'a, Storage, K> {
	/// Creates a new instance of the [`For`] for expression storage.
	pub fn new(factory: impl 'static + FnMut() -> Result<Storage, Escalation>) -> Self {
		Self::new_(Box::new(factory))
	}

	fn new_(factory: Box<dyn 'static + FnMut() -> Result<Storage, Escalation>>) -> Self {
		Self {
			linotype: Linotype::new().pin(),
			factory,
		}
	}

	/// Implementation detail.
	#[doc(hidden)]
	#[allow(clippy::type_complexity, non_snake_case)]
	pub fn __Asteracea__update_try_by<'b, 'c: 'b, T, Q: 'b>(
		&'b mut self,
		items: impl 'c + IntoIterator<Item = T>,
		selector: impl 'c + FnMut(&mut T) -> Result<&Q, Escalation>,
	) -> Box<dyn 'b + Iterator<Item = Result<(T, Pin<&mut Storage>), Escalation>>>
	where
		T: 'b,
		K: Borrow<Q>,
		Q: Eq + ToOwned<Owned = K>,
	{
		let factory = &mut self.factory;
		<Pin<Linotype<K, Storage>> as PinningLinotype>::update_try_by_try_with(
			&mut self.linotype,
			items,
			selector,
			move |_| factory(),
		)
	}
}

pub struct MixedKey {
	// variant: MixedKey_,
}

// #[allow(incorrect_ident_case)]
// enum MixedKey_ {
// 	char(char),
// 	i8(i8),
// 	i16(i16),
// 	i32(i32),
// 	i64(i64),
// 	i128(i128),
// 	isize(isize),
// 	u8(u8),
// 	u16(u16),
// 	u32(u32),
// 	u64(u64),
// 	u128(u128),
// 	usize(usize),
// 	f32(f32),
// 	f64(f64),
// 	bool(bool),
// 	String(String),
// }
