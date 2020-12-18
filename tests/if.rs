use asteracea::lignin_schema::lignin::{bumpalo::Bump, Node};

asteracea::component! {
	pub Conditional1()(
		present: bool,
	)

	if {present}
		"I am here."
}

asteracea::component! {
	pub Conditional2()(
		present: bool,
	)

	[
		if {present} [ // <-- I recommend formatting this `[]` as you would format `{}` in Rust.
			"I am here"
			<span " and ">
		]
		"I like this place."
	]
}

asteracea::component! {
	pub Conditional3()(
		content?: impl for<'b> FnOnce(&'b Bump) -> Node<'b>,
	)

	[
		"["
		if {let Some(content) = content}
			{ content(bump) }
		"]"
	]
}
