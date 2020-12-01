use asteracea::component;

component! {
	Attributed()()
		<div
			."id" = "Hello!"
			."class" = {"a-class"}
	>
}

component! {
	SometimesAttributes()(
		class: Option<&'bump str>,
	)

	<div
		."class"? = {class}
	>
}

//TODO: Test output.
