asteracea::component! { substrate =>
	pub NewArgPattern(
		ab @ (_a, _b): (usize, usize)
	)() -> Sync

	[]
}

asteracea::component! { substrate =>
	pub RenderArgPattern()(
		ab @ (_a, _b): (usize, usize)
	) -> Sync

	[]
}
