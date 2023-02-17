use asteracea::substrates::web as substrate;
use bumpalo::Bump;
use lignin::{Node, ThreadBound};

asteracea::component! { substrate =>
	pub Conditional1()(
		present: bool,
	)

	spread if {present}
		"I am here."
}

asteracea::component! { substrate =>
	pub Conditional2()(
		present: bool,
	)

	[
		spread if {present} [ // <-- I recommend formatting this `[]` as you would format `{}` in Rust.
			"I am here"
			<span " and ">
		]
		"I like this place."
	]
}

asteracea::component! { substrate =>
	pub Conditional3()(
		content?: impl for<'b> FnOnce(&'b Bump) -> Node<'b, ThreadBound>,
	)

	[
		"["
		spread if {let Some(content) = content}
			{ content(bump) }
		"]"
	]
}
