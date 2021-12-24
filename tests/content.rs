asteracea::component! {
	Container()(..)

	<"custom-container"
		..
	>
}

asteracea::component! {
	Content()()

	<"custom-content">
}

asteracea::component! {
	Parent()()

	<*Container
		<*Content>
	>
}
