#[allow(dead_code)]

asteracea::component! {
	Child()()

	[]
}

asteracea::component! {
	Parent()()

	<*Child>
}
