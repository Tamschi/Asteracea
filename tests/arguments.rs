use asteracea::component;

component! {
	Greeting()(
		greeting: &str = "Hello!",
	)

	<span
		."class" = "greeting"
		!{greeting}
	>
}

asteracea::component! {
	Classic()(
		class?: &'bump str,
	)

	<div
		."class"? = {class} // `Option<_>`-typed!
	>
}

// TODO: Test output.
