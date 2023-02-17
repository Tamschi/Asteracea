use asteracea::substrates::web as substrate;

asteracea::component! { substrate =>
	pub Greeting()(
		user: &'bump str = "and welcome to web gardening"
	)

	!"Hello, {}!"(user)
}
