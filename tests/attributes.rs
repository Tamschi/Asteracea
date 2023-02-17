#![allow(dead_code)] //TODO: Test output.

asteracea::component! { substrate =>
	Attributed()() -> Sync
	<div
		.id = "Hello!"
		.class = {"a-class"}
	>
}

asteracea::component! { substrate =>
	SometimesAttributes()(
		class: Option<&'bump str>,
	)

	<div
		.class? = {class}
	>
}

asteracea::component! { substrate =>
	MultiOptional()(
		class: Option<&'bump str>,
	)

	<div
		."a" = ""
		.class? = {class}
		."class2"? = {class}
		."b" = ""
		."c" = ""
	>
}

asteracea::component! { substrate =>
  Vis()(
	visible: bool,
  )

  <div
	.hidden? = {!visible}
	.role = ""
	"#"
  >
}
