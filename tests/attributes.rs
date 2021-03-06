#![allow(dead_code)] //TODO: Test output.

use asteracea::component;

component! {
	Attributed()() -> Sync
	<div
		.id = "Hello!"
		.class = {"a-class"}
	>
}

component! {
	SometimesAttributes()(
		class: Option<&'bump str>,
	)

	<div
		.class? = {class}
	>
}

component! {
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

asteracea::component! {
  Vis()(
	visible: bool,
  )

  <div
	.hidden? = {!visible}
	.role = ""
	"#"
  >
}
