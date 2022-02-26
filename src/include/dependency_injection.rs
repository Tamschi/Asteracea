use rhizome::sync::{DynValue, Node, NodeHandle};
use std::{any::TypeId, pin::Pin};

pub type ResourceNode = Node<TypeId, TypeId, DynValue>;
pub type ResourceNodeHandle = NodeHandle<TypeId, TypeId, DynValue>;

enum ParentOrBranched<'a> {
	Parent(Pin<&'a ResourceNode>),
	Branched(ResourceNodeHandle),
}

#[must_use = "This should be passed further along as `SparseNode`."]
pub struct BranchOnBorrow<'a> {
	tag: TypeId,
	state: ParentOrBranched<'a>,
}

impl<'a> BranchOnBorrow<'a> {
	pub fn new_for<T: 'static>(parent: Pin<&'a ResourceNode>) -> Self {
		Self::new(TypeId::of::<T>(), parent)
	}

	pub fn new(tag: TypeId, parent: Pin<&'a ResourceNode>) -> Self {
		Self {
			tag,
			state: ParentOrBranched::Parent(parent),
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
	pub fn into_sparse_node(self) -> SparseNode<'a> {
		SparseNode { value: self.state }
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
