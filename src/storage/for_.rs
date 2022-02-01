use crate::error::Escalation;
use core::mem;
use linotype::{OwnedProjection, PinningOwnedProjection};
use std::{
	borrow::Borrow,
	collections::hash_map::RandomState,
	hash::{BuildHasher, Hasher},
	pin::Pin,
};

/// Storage for [`for`](`For`) expressions.
///
/// > BUG:
/// >
/// > Instead of storing only Storage, this needs to also store a [`u32`] to set up a [`lignin::Node::Keyed`].
/// >
/// > Right now, a plain [`lignin::Node::Multi`] is generated, which means internal component state is affixed properly while external component state is not.
/// > (The unbinding and binding functionality still runs appropriately, so this doesn't cause extremely severe issues, but it can lead to UX degradation in many cases.)
pub struct For<'a, Storage, K = MixedKey, S = RandomState> {
	build_hasher: S,
	storage: Pin<OwnedProjection<K, Reorderable<Storage>>>,
	factory: Box<dyn 'a + FnMut() -> Result<Storage, Escalation>>,
}

impl<'a, Storage, K, S> For<'a, Storage, K, S> {
	/// Creates a new instance of the [`For`] for expression storage.
	pub fn new(factory: impl 'static + FnMut() -> Result<Storage, Escalation>) -> Self
	where
		S: Default,
	{
		Self::new_(Box::new(factory))
	}

	fn new_(factory: Box<dyn 'static + FnMut() -> Result<Storage, Escalation>>) -> Self
	where
		S: Default,
	{
		Self {
			build_hasher: S::default(),
			storage: OwnedProjection::new().pin(),
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
	) -> Box<dyn 'b + Iterator<Item = Result<(T, Pin<&mut Reorderable<Storage>>), Escalation>>>
	where
		T: 'b,
		K: Borrow<Q> + ReprojectionKey,
		Q: Eq + ToOwned<Owned = K>,
		S: BuildHasher,
	{
		let factory = &mut self.factory;
		let hasher = &self.build_hasher;
		self.storage
			.reproject_try_by_try_with(items, selector, move |_item, k| {
				Ok(Reorderable {
					dom_key: ReprojectionKey::to_dom_key(k, hasher),
					storage: factory()?,
				})
			})
	}
}

pub struct MixedKey {
	// variant: MixedKey_,
}

impl ReprojectionKey for MixedKey {
	fn to_dom_key(&self, _build_hasher: &impl BuildHasher) -> u32 {
		todo!()
	}
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

pub struct Reorderable<Storage> {
	pub dom_key: u32,
	storage: Storage,
}

impl<Storage> Reorderable<Storage> {
	pub fn storage(self: Pin<&Self>) -> Pin<&Storage> {
		unsafe { self.map_unchecked(|this| &this.storage) }
	}
}

pub trait ReprojectionKey {
	fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32;
}

// TODO: Reevaluate use a hasher here. It works, but it's probably not great in terms of speed.

macro_rules! impl_reprojection_keys_abs {
	($($type:ty),*$(,)?) => {$(
		impl ReprojectionKey for $type {
			fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
				const _: () = {
					assert!(mem::size_of::<$type>() <= mem::size_of::<u32>())
				};
				let mut hasher = build_hasher.build_hasher();
				hasher.write_u8(0); // TODO: Is this necessary?
				let base = hasher.finish() as u32;
				base.wrapping_add((*self).wrapping_abs() as u32)
			}
		}
	)*};
}

impl_reprojection_keys_abs!(i8, i16, i32);

macro_rules! impl_reprojection_keys {
	($($type:ty),*$(,)?) => {$(
		impl ReprojectionKey for $type {
			fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
				const _: () = {
					assert!(mem::size_of::<$type>() <= mem::size_of::<u32>())
				};
				let mut hasher = build_hasher.build_hasher();
				hasher.write_u8(0);
				let base = hasher.finish() as u32;
				base.wrapping_add(*self as u32)
			}
		}
	)*};
}

impl_reprojection_keys!(bool, char, u8, u16, u32);

impl ReprojectionKey for u64 {
	fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
		let mut hasher = build_hasher.build_hasher();
		hasher.write_u64(*self);
		#[allow(clippy::cast_possible_truncation)]
		let dom_key = hasher.finish() as u32;
		dom_key
	}
}

impl ReprojectionKey for u128 {
	fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
		let mut hasher = build_hasher.build_hasher();
		hasher.write_u128(*self);
		#[allow(clippy::cast_possible_truncation)]
		let dom_key = hasher.finish() as u32;
		dom_key
	}
}

impl ReprojectionKey for i64 {
	fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
		let mut hasher = build_hasher.build_hasher();
		hasher.write_i64(*self);
		#[allow(clippy::cast_possible_truncation)]
		let dom_key = hasher.finish() as u32;
		dom_key
	}
}

impl ReprojectionKey for i128 {
	fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
		let mut hasher = build_hasher.build_hasher();
		hasher.write_i128(*self);
		#[allow(clippy::cast_possible_truncation)]
		let dom_key = hasher.finish() as u32;
		dom_key
	}
}

impl ReprojectionKey for String {
	fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
		let mut hasher = build_hasher.build_hasher();
		hasher.write(self.as_bytes());
		#[allow(clippy::cast_possible_truncation)]
		let dom_key = hasher.finish() as u32;
		dom_key
	}
}
