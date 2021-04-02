use asteracea::component;
use bumpalo::Bump;
use rhizome::Node;

component! {
	pub Greeting()(
		greeting: &str = "Hello!",
	) -> Sync

	<span
		."class" = "greeting"
		!{greeting}
	>
}

asteracea::component! {
	pub Classic()(
		class?: &'bump str,
	) -> Sync

	<div
		."class"? = {class} // `Option<_>`-typed!
	>
}

asteracea::component! {
  Inner()(
	class?: &'bump str,
  )

  <span ."class"? = {class}>
}

asteracea::component! {
  Middle()(
	class?: &'bump str,
  )

  <*Inner .class? = {class}>
}

asteracea::component! {
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
			&Node::new_for::<()>().into_arc(),
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
