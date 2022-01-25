//! Temporary module pending extraction into its own library.
#![allow(missing_docs)]

use core::any::TypeId;
use fruit_salad::Dyncast;
use rhizome::sync::{Node, NodeHandle};
use std::pin::Pin;

pub type ResourceNode = Node<TypeId, TypeId, dyn Dyncast>;
pub type ResourceNodeHandle = NodeHandle<TypeId, TypeId, dyn Dyncast>;

/// Marker trait. These types can act as keys for dependency injection.
pub trait InjectionKey
where
	Self: 'static,
{
}

/// Helper trait to extract values and references from [`Node`]s.
pub trait Extract {}
impl<T, K: Ord, V: ?Sized> Extract for Node<T, K, V> {}

#[derive(Debug, Dyncast)]
#[dyncast(#![runtime_pointer_size_assertion] unsafe T)]
#[repr(transparent)]
pub struct OwnedValueProvider<T>(pub T);

pub trait Factory<T>: Dyncast {
	fn produce(self: Pin<&Self>) -> T;
}

#[derive(Debug, Dyncast)]
#[dyncast(#![runtime_pointer_size_assertion] dyn Factory<T>)]
pub struct FnFactory<T: 'static, F: 'static + Fn() -> T>(F);

impl<T, F: Fn() -> T> Factory<T> for FnFactory<T, F> {
	fn produce(self: Pin<&Self>) -> T {
		(self.0)()
	}
}
