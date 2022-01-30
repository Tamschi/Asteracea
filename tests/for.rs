asteracea::component! {
	pub ForImplicit()() -> Sync

	for i: u8 in 1..=5, !"{}"(i)
}

asteracea::component! {
	pub ForExplicit()() -> Sync

	for i: u8 keyed i => u8 in [1, 2, 3, 4, 5], !"{}"(i)
}

// asteracea::component! {
// 	pub ForUntyped()() -> Sync

// 	for i: u8 keyed i in [1, 2, 3, 4, 5], !"{}"(i)
// }
