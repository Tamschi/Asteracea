use asteracea::substrates::web as substrate;

asteracea::component! { substrate =>
	pub NewArgPattern(
		ab @ (_a, _b): (usize, usize)
	)()

	[]
}

asteracea::component! { substrate =>
	pub RenderArgPattern()(
		ab @ (_a, _b): (usize, usize)
	)

	[]
}
