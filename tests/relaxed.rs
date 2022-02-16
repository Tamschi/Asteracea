use asteracea::components::Remember;

asteracea::component! {
	FrequentlyUnchanged()()

	<*Remember
		// .for_unchanged={&()}
		.or_unless={|| false}
		"Content!"
	>
}
