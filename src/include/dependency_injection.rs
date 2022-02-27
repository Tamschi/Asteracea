use rhizome::sync::{DynValue, Node, NodeHandle};
use std::{any::TypeId, marker::PhantomPinned, ops::Deref, pin::Pin};

pub type ResourceNode = Node<TypeId, TypeId, DynValue>;
pub type ResourceNodeHandle = NodeHandle<TypeId, TypeId, DynValue>;

enum ParentOrBranched<'a> {
	Parent(Pin<&'a ResourceNode>),
	Branched(ResourceNodeHandle),
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
	state: ParentOrBranched<'a>,
	_pinned: PhantomPinned,
}

impl<'a> ResourceBob<'a> {
	pub fn new_for<T: 'static>(parent: Pin<&'a ResourceNode>) -> Self {
		Self::new(TypeId::of::<T>(), parent)
	}

	pub fn new(tag: TypeId, parent: Pin<&'a ResourceNode>) -> Self {
		Self {
			tag,
			state: ParentOrBranched::Parent(parent),
			_pinned: PhantomPinned,
		}
	}

	#[must_use = "This operation is usually not free, so discarding the result is usually not what you want. If you just want to make sure a node is created, you can discard explicitly."]
	pub fn borrow(self: Pin<&mut Self>) -> Pin<&ResourceNode> {
		let this = unsafe { Pin::into_inner_unchecked(self) };

		if let ParentOrBranched::Parent(parent) = this.state {
			this.state = ParentOrBranched::Branched(parent.branch_for(this.tag));
		}

		match &this.state {
			ParentOrBranched::Parent(_) => unreachable!(),
			ParentOrBranched::Branched(branched) => branched.as_ref(),
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
	value: ParentOrBranched<'a>,
}

impl SparseResourceNodeHandle<'_> {
	pub fn as_ref(&self) -> Pin<&ResourceNode> {
		match &self.value {
			ParentOrBranched::Parent(ref_) => *ref_,
			ParentOrBranched::Branched(handle) => handle.as_ref(),
		}
	}
}
