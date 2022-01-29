asteracea::component! {
	pub For()() -> Sync

	for i keyed &i => u8 in [1, 2, 3, 4, 5], !"{}"(i)
}
