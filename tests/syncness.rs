asteracea::component! {
	pub Sync()() -> Sync

	[]
}

asteracea::component! {
	pub UnSync()() -> !Sync

	[]
}

asteracea::component! {
	#[allow(dead_code)]
	AutoSync()()

	[]
}

asteracea::component! {
	pub ExplicitAutoSync()() -> Sync?

	[]
}
