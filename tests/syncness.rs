asteracea::component! { substrate =>
	pub Sync()() -> Sync

	[]
}

asteracea::component! { substrate =>
	pub UnSync()() -> !Sync

	[]
}

asteracea::component! { substrate =>
	#[allow(dead_code)]
	AutoSync()()

	[]
}

asteracea::component! { substrate =>
	pub ExplicitAutoSync()() -> Sync?

	[]
}
