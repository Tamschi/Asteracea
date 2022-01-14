asteracea::component! {
	pub NewArgPattern(
		ab @ (_a, _b): (usize, usize)
	)() -> Sync

	[]
}

asteracea::component! {
	pub RenderArgPattern()(
		ab @ (_a, _b): (usize, usize)
	) -> Sync

	[]
}
