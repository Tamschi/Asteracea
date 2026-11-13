mod _sub;

use std::any::TypeId;

use bumpalo::Bump;
use rhizome::sync::Node;

asteracea::components! {
	#![substrate = _sub::Sub]

	pub Greeting[greeting: &'_ str = "Hello!"] => span["class" = "greeting"] {
		"{greeting}"[];
	}

	pub Classic[class?: &'_ str] => div["class"? = class]; // `Option<_>`-typed!

	Inner[class?: &'_ str] => span["class"? = class];
	Middle[class?: &'_ str] => Inner[class? = class];
	Outer => {
		Middle; "\n";
		Middle[class = "bourgeoisie"];
	}
}

#[test]
pub(crate) fn test() {
	let outer = Box::pin(
		Outer::new(
			Node::new(TypeId::of::<()>()).as_ref(),
			OuterInitArgs::builder().build(),
		)
		.unwrap(),
	);
	outer
		.as_ref()
		.render(&Bump::new(), OuterRenderArgs::builder().build())
		.unwrap();

	// TODO: Test output.
}
