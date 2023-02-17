use asteracea::substrates::web as substrate;

asteracea::component! { substrate =>
	pub Sync()()

	[]
}

asteracea::component! { substrate =>
	pub UnSync()()

	[]
}

asteracea::component! { substrate =>
	#[allow(dead_code)]
	AutoSync()()

	[]
}

asteracea::component! { substrate =>
	pub ExplicitAutoSync()()

	[]
}
