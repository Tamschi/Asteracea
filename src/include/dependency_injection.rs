use rhizome::sync::{DynValue, Node, NodeHandle};
use std::{any::TypeId, marker::PhantomPinned, ops::Deref, pin::Pin};

pub type ResourceNode = Node<TypeId, TypeId, DynValue>;
pub type ResourceNodeHandle = NodeHandle<TypeId, TypeId, DynValue>;

enum ParentOrOwned<'a> {
	Parent(Pin<&'a ResourceNode>),
	Owned(ResourceNodeHandle),
}

/// A "branch on borrow" handle.
///
/// This is exposed, by reference, to constructor-scoped user code in Asteracea components,
/// and can be used to inject resources after borrowing from it.
///
/// If the latter is not done, then no resource node is created for the current component.
#[must_use = "This should be passed further along as `SparseResourceNode`."]
pub struct ResourceBob<'a> {
	tag: TypeId,
	state: ParentOrOwned<'a>,
	_pinned: PhantomPinned,
}

impl<'a> ResourceBob<'a> {
	pub fn new_for<T: 'static>(parent: Pin<&'a ResourceNode>) -> Self {
		Self::new(TypeId::of::<T>(), parent)
	}

	pub fn new(tag: TypeId, parent: Pin<&'a ResourceNode>) -> Self {
		Self {
			tag,
			state: ParentOrOwned::Parent(parent),
			_pinned: PhantomPinned,
		}
	}

	#[must_use = "This operation is usually not free, so discarding the result is usually not what you want. If you just want to make sure a node is created, you can discard explicitly."]
	pub fn borrow(self: Pin<&mut Self>) -> Pin<&ResourceNode> {
		let this = unsafe { Pin::into_inner_unchecked(self) };

		if let ParentOrOwned::Parent(parent) = this.state {
			this.state = ParentOrOwned::Owned(parent.branch_for(this.tag));
		}

		match &this.state {
			ParentOrOwned::Parent(_) => unreachable!(),
			ParentOrOwned::Owned(branched) => branched.as_ref(),
		}
	}

	#[must_use]
	pub fn into_sparse_node_handle(self) -> SparseResourceNodeHandle<'a> {
		SparseResourceNodeHandle { value: self.state }
	}
}

/// Returned as second value from each Asteracea component constructor.
///
/// The lifetime `'a` matches that of the parent [`&ResourceNode`](`ResourceNode`),
/// and indeed that reference may be contained here directly.
pub struct SparseResourceNodeHandle<'a> {
	value: ParentOrOwned<'a>,
}

impl SparseResourceNodeHandle<'_> {
	pub fn as_ref(&self) -> Pin<&ResourceNode> {
		match &self.value {
			ParentOrOwned::Parent(ref_) => *ref_,
			ParentOrOwned::Owned(handle) => handle.as_ref(),
		}
	}

	pub fn into_owned(self) -> ResourceNodeHandle {
		match self.value {
			ParentOrOwned::Parent(parent) => parent.clone_handle(),
			ParentOrOwned::Owned(handle) => handle,
		}
	}
}
