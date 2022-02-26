use rhizome::sync::{DynValue, Node, NodeHandle};
use std::{
	any::TypeId,
	borrow::BorrowMut,
	cell::{RefCell, UnsafeCell},
	ops::Deref,
	pin::Pin,
};

pub type ResourceNode = Node<TypeId, TypeId, DynValue>;
pub type ResourceNodeHandle = NodeHandle<TypeId, TypeId, DynValue>;

enum ParentOrBranched<'a> {
	Parent(Pin<&'a ResourceNode>),
	Branched(ResourceNodeHandle),
}

pub struct BranchOnBorrow<'a> {
	tag: TypeId,
	state: UnsafeCell<ParentOrBranched<'a>>,
}

impl<'a> BranchOnBorrow<'a> {
	pub fn new_for<T: 'static>(parent: Pin<&'a ResourceNode>) -> Self {
		Self::new(TypeId::of::<T>(), parent)
	}

	pub fn new(tag: TypeId, parent: Pin<&'a ResourceNode>) -> Self {
		Self {
			tag,
			state: ParentOrBranched::Parent(parent).into(),
		}
	}

	#[must_use = "This operation is usually not free, so discarding the result is usually not what you want. If you just want to make sure a node is created, you can discard explicitly."]
	pub fn borrow(self: Pin<&mut Self>) -> Pin<&ResourceNode> {
		unsafe {
			if let ParentOrBranched::Parent(parent) = &*self.state.get() {
				*self.state.get() = ParentOrBranched::Branched(parent.branch_for(self.tag));
			}
		}

		match unsafe { &*self.state.get() } {
			ParentOrBranched::Parent(parent) => *parent,
			ParentOrBranched::Branched(branched) => branched.as_ref(),
		}
	}

	pub fn into_sparse_node(self) -> SparseNode<'a> {
		SparseNode {
			value: self.state.into_inner(),
		}
	}
}

pub struct SparseNode<'a> {
	value: ParentOrBranched<'a>,
}

impl SparseNode<'_> {
	pub fn as_ref(&self) -> Pin<&ResourceNode> {
		match &self.value {
			ParentOrBranched::Parent(ref_) => *ref_,
			ParentOrBranched::Branched(handle) => handle.as_ref(),
		}
	}
}
