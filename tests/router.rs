use asteracea::components::Router;
use core::cell::Cell;

asteracea::component! {
	pub RouterTester()(
		path: &'bump str,
	) -> Sync

	with {
		let rest = Cell::default();
	} <*Router
		.path={path}
		.rest={&rest}
		<div ^path={"/div/"}
			!"{}"{rest.get()}
		>
		<span ^path={"/span/"}
			!"{}"{rest.get()}
		>
	/Router>
}
