use asteracea::components::Router;

asteracea::component! {
	pub RouterTester()(
		path: &'bump str,
	) -> Sync

	with {
		let rest = Cell::default;
	} <*Router
		.path={path}
		.rest={&rest}
		<div ^path={"/div/"}
			!"{}"{rest.get().unwrap()}
		>
		<span ^path={"/span/"}
			!"{}"{rest.get().unwrap()}
		>
	/Router>
}
