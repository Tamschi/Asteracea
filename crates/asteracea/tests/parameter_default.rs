asteracea::component! {
	pub Greeting()(
		user: &'bump str = "and welcome to web gardening"
	) -> Sync

	!"Hello, {}!"(user)
}
