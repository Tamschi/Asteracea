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

// TODO: Test output.
