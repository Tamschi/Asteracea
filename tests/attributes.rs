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

component! {
	MultiOptional()(
		class: Option<&'bump str>,
	)

	<div
		."a" = ""
		."class"? = {class}
		."class2"? = {class}
		."b" = ""
		."c" = ""
	>
}

//TODO: Test output.
