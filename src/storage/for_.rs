use crate::error::Escalation;
use core::{hash::Hash, mem};
use linotype::{OwnedProjection, PinningOwnedProjection};
use std::{
	any::{Any, TypeId},
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
pub struct For<'a, Storage, K = BoxedAnyOneK, St = RandomState> {
	build_hasher: St,
	storage: Pin<OwnedProjection<K, Reorderable<Storage>>>,
	factory: Box<dyn 'a + FnMut() -> Result<Storage, Escalation>>,
}

impl<'a, Storage, K, St> For<'a, Storage, K, St> {
	/// Creates a new instance of the [`For`] for expression storage.
	pub fn new(factory: impl 'static + FnMut() -> Result<Storage, Escalation>) -> Self
	where
		St: Default,
	{
		Self::new_(Box::new(factory))
	}

	fn new_(factory: Box<dyn 'static + FnMut() -> Result<Storage, Escalation>>) -> Self
	where
		St: Default,
	{
		Self {
			build_hasher: St::default(),
			storage: OwnedProjection::new().pin(),
			factory,
		}
	}

	/// Implementation detail.
	#[doc(hidden)]
	#[allow(clippy::type_complexity, non_snake_case)]
	pub fn __Asteracea__reproject_try_by<'b, 'c: 'b, T, Q, I, S>(
		&'b mut self,
		items: I,
		selector: S,
	) -> Box<dyn 'b + Iterator<Item = Result<(T, Pin<&mut Reorderable<Storage>>), Escalation>>>
	where
		K: Borrow<Q> + ReprojectionKey,
		St: BuildHasher,
		T: 'b,
		Q: 'b + ?Sized + Eq + ToOwned<Owned = K>,
		I: 'c + IntoIterator<Item = T>,
		S: 'c + FnMut(&mut T) -> Result<&Q, Escalation>,
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

pub struct BoxedAnyOneK {
	// These are cached, so it's not TOO horrible, but it's still inefficient.
	dynamic: Box<dyn DynamicReprojectionKey>,
}

/// # Safety
///
/// `.downcast_ref_(…)` must assert that the given [`TypeId`] is that of the target of the returned pointer.
unsafe trait DynamicReprojectionKey: Any {
	fn downcast_ref_(&self, type_id: TypeId) -> *const ();
	fn hash(&self, state: &mut dyn Hasher);
}
impl dyn DynamicReprojectionKey {
	fn downcast_ref<T: 'static>(&self) -> &T {
		let ptr = self.downcast_ref_(TypeId::of::<T>()).cast::<T>();
		unsafe { &*(ptr) }
	}
}

unsafe impl<T> DynamicReprojectionKey for T
where
	T: Any + Hash,
{
	fn downcast_ref_(&self, type_id: TypeId) -> *const () {
		assert_eq!(type_id, TypeId::of::<Self>(), "Attempted to borrow `BoxedAnyOneK` as mismatching `InferredQ`: This construct is a workaround pending better type inference and doesn't support mixing stored key types.");
		(self as *const Self).cast::<()>()
	}

	fn hash(&self, mut state: &mut dyn Hasher) {
		Hash::hash(&self, &mut state)
	}
}

impl<Q: ?Sized> Borrow<InferredQ<Q>> for BoxedAnyOneK
where
	Q: ToOwned,
	Q::Owned: 'static,
{
	fn borrow(&self) -> &InferredQ<Q> {
		let q = self
			.dynamic
			.downcast_ref::<<Q as ToOwned>::Owned>()
			.borrow();
		InferredQ::from_ref(q)
	}
}

#[derive(PartialEq, Eq)]
pub struct InferredQ<Q: ?Sized>(Q);

impl<Q: ?Sized> InferredQ<Q> {
	pub fn from_ref(q: &Q) -> &Self {
		unsafe {
			//SAFETY: `Self` is transparent towards `Q`.
			&*(q as *const _ as *const _)
		}
	}
}

impl<Q: ?Sized> ToOwned for InferredQ<Q>
where
	Q: ToOwned,
	Q::Owned: 'static + Hash,
{
	type Owned = BoxedAnyOneK;

	fn to_owned(&self) -> Self::Owned {
		BoxedAnyOneK {
			dynamic: Box::new(self.0.to_owned()),
		}
	}
}

impl ReprojectionKey for BoxedAnyOneK {
	// Not an exact science.
	// There may be spurious GUI rebuilds,
	// though no inconsistency except for very rare focus shifts and such,
	// and only when the input sequence is really being changed.
	#[allow(clippy::cast_possible_truncation)]
	fn to_dom_key(&self, build_hasher: &impl BuildHasher) -> u32 {
		let mut hasher = build_hasher.build_hasher();
		self.dynamic.hash(&mut hasher);

		hasher.finish() as u32
	}
}

pub struct Reorderable<Storage> {
	pub dom_key: u32,
	storage: Storage,
}

impl<Storage> Reorderable<Storage> {
	#[must_use]
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
