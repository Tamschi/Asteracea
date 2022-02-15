use asteracea::components::Paused;

asteracea::component! {
	FrequentlyUnchanged()()

	<*Relaxed
		// .for_unchanged={&()}
		.unless={|| false}
		"Content!"
	>
}
