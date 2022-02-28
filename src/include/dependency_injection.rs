//! Sparsely-branching dependency injection infrastructure.
//!
//! This module is used by all Asteracea-generated components, but by itself is very lightweight.
//! Its main function is to avoid unnecessary heap allocations, in addition to the sparse reference-counting already enabled by [`rhizome`]'s [`tiptoe`] dependency.

use rhizome::sync::{DynValue, Node, NodeHandle};
use std::{any::TypeId, marker::PhantomPinned, pin::Pin};

/// A [heap-only](https://blog.schichler.dev/posts/Intrusive-Smart-Pointers-+-Heap-Only-Types-=/)
/// resource node used for dependency injection.
pub type ResourceNode = Node<TypeId, TypeId, DynValue>;

/// A fully owned and thread-safe handle to a [`ResourceNode`] used for dependency injection.
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
	/// Creates a new instance of [`ResourceNode`] that, on branching, will tag the branch with `T`'s [`TypeId`].
	pub fn new_for<T: 'static>(parent: Pin<&'a ResourceNode>) -> Self {
		Self::new(TypeId::of::<T>(), parent)
	}

	/// Creates a new instance of [`ResourceNode`] that, on branching, will tag the branch with `tag`.
	pub fn new(tag: TypeId, parent: Pin<&'a ResourceNode>) -> Self {
		Self {
			tag,
			state: ParentOrBranched::Parent(parent),
			_pinned: PhantomPinned,
		}
	}

	/// Unconditionally ensures there is an owned resource node branch in order to decouple its lifetime (e.g. for async component constructors).
	pub fn into_owned(self) -> ResourceBob<'static> {
		ResourceBob {
			tag: self.tag,
			state: ParentOrBranched::Branched(match self.state {
				ParentOrBranched::Parent(parent) => parent.branch_for(self.tag),
				ParentOrBranched::Branched(owned) => owned,
			}),
			_pinned: PhantomPinned,
		}
	}

	/// Borrows the underlying *local branch* [`ResourceNode`], creating it if necessary.
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

	/// Converts this [`ResourceBob`] into a [`SparseResourceNodeHandle`], which can be borrowed without branching.
	#[must_use]
	pub fn into_sparse_handle(self) -> SparseResourceNodeHandle<'a> {
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
	/// Retrieves a pinning reference to the underlying [`ResourceNode`].
	#[must_use]
	pub fn as_ref(&self) -> Pin<&ResourceNode> {
		match &self.value {
			ParentOrBranched::Parent(ref_) => *ref_,
			ParentOrBranched::Branched(handle) => handle.as_ref(),
		}
	}
}
