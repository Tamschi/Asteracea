use asteracea::error::ExtractableResolutionError;
use lignin::bumpalo::Bump;
use rhizome::Node;

asteracea::component! {
	Deferred()() []
}

asteracea::component! {
	Deferrer()()

	[
		//TODO
		// defer <*Deferred>

		// with {
		// 	assert_eq!(self.deferred_0, None, "First named `defer` initialised too early")
		// } defer as defer_0 <*Deferred as deferred_0>

		// with {
		// 	if let Some(defer) = self.defer_0.as_ref() {
		// 		let _: &Deferred = &defer.deferred_0;
		// 	} else {
		// 		panic!("First named `defer` not initialised")
		// 	}

		// 	assert_eq!(self.defer_1, None, "Second named `defer` initialised too early")
		// } defer as deferred_1: struct DeferContainer <*Deferred>

		// with {
		// 	if let Some(defer) = self.defer_1.as_ref() {
		// 		let defer: &DeferContainer = defer;
		// 		let _: &Deferred = &defer.deferred_1;
		// 	} else {
		// 		panic!("Second named `defer` not initialised")
		// 	}
		// } []
	]
}

#[test]
fn defer() -> Result<(), ExtractableResolutionError> {
	let root = Node::new_for::<()>();
	let component = Deferrer::new(&root.into(), Deferrer::new_args_builder().build())?;

	let bump = Bump::new();
	let _vdom = component.render(&bump, Deferrer::render_args_builder().build());

	Ok(())
}
