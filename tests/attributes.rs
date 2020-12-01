use asteracea::component;

component! {
    Attributed()()
    <div
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