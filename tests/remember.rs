use asteracea::components::Remember;
use lignin::ThreadSafe;

asteracea::component! {
	FrequentlyUnchanged()()

	<*Remember::<ThreadSafe>
		// .for_unchanged={&()}
		.or_unless={|| false}
		"Content!"
	>
}
