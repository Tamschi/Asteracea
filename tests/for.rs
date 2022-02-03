asteracea::component! {
	pub ForImplicit()() -> Sync

	for i in [1, 2, 3, 4, 5i32] {
	 !"{}"(i)
	}
}

asteracea::component! {
	pub ForImplicitSelector()() -> Sync

	for i: u8 in 1..=5 {
		!"{}"(i)
	}
}

asteracea::component! {
	pub ForImplicitItemType()() -> Sync

	for i keyed i => u8 in 1..=5 {
		!"{}"(i)
	}
}

asteracea::component! {
	pub ForKeyTypeOnly()() -> Sync

	for i => u8 in &[1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! {
	pub ForExplicit()() -> Sync

	for i: u8 keyed i => u8 in [1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! {
	pub ForUntyped()() -> Sync

	for i keyed i in [1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! {
  pub Split()() -> Sync

  for c in "This is a test.".split(' ') {[
	  <li
		!"{:?}"(c)
	  > "\n"
  ]}
}

asteracea::component! {
  pub Child()() -> Sync

  for _ in 0..5 {
	  <*ForImplicit>
  }
}
