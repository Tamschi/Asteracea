use asteracea::substrates::web as substrate;
use bumpalo::Bump;
use rhizome::sync::Node;
use std::any::TypeId;

asteracea::component! { substrate =>
	pub Greeting()(
		greeting: &str = "Hello!",
	)

	<span
		."class" = "greeting"
		!(greeting)
	>
}

asteracea::component! { substrate =>
	pub Classic()(
		class?: &'bump str,
	)

	<div
		."class"? = {class} // `Option<_>`-typed!
	>
}

asteracea::component! { substrate =>
  Inner()(
	class?: &'bump str,
  )

  <span ."class"? = {class}>
}

asteracea::component! { substrate =>
  Middle()(
	class?: &'bump str,
  )

  <*Inner .class? = {class}>
}

asteracea::component! { substrate =>
  Outer()()

  [
	<*Middle> "\n"
	<*Middle .class = {"bourgeoisie"}>
  ]
}

#[test]
fn test() {
	let outer = Box::pin(
		Outer::new(
			Node::new(TypeId::of::<()>()).as_ref(),
			Outer::new_args_builder().build(),
		)
		.unwrap(),
	);
	outer
		.as_ref()
		.render(&Bump::new(), Outer::render_args_builder().build())
		.unwrap();

	// TODO: Test output.
}
