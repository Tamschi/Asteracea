#[allow(dead_code)]

asteracea::component! {
	Child()() -> Sync

	[]
}

asteracea::component! {
	Parent()()

	<*Child>
}
