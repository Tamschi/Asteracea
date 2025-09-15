use pinned_init::PinInit;

pub trait Substrate {
	type InjectionNode;
}

pub trait Component<S: Substrate> {
	type NewArgs;

	fn new(parent_node: S::InjectionNode, args: Self::NewArgs) -> impl PinInit<Self>;
}
