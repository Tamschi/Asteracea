//! Temporary module pending extraction into its own library.
#![allow(missing_docs)]

use core::any::TypeId;
use fruit_salad::Dyncast;
use rhizome::sync::{Node, NodeHandle};

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
