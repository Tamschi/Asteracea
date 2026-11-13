mod _sub;

asteracea::components! {
	#![substrate = _sub::Sub]

	pub Conditional1[
		present: bool,
	] => [if] present {
		"I am here.";
	}

	pub Conditional2[
		present: bool,
	] => {
		[if] present {
			"I am here";
			span{ " and "; }
		}
		"I like this place.";
	}

	pub Conditional3[
		content?: impl for<'b> FnOnce(&'b Bump) -> Node<'b, ThreadBound>,
	] => {
		"[";
		[if] let Some(content) = content {
			[{ content(bump) }]
		}
		"]";
	}
}
