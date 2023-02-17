asteracea::component! { substrate =>
	pub Greeting()(
		user: &'bump str = "and welcome to web gardening"
	) -> Sync

	!"Hello, {}!"(user)
}
