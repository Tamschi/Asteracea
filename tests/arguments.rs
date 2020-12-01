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

// TODO: Test output.
