asteracea::component! { substrate =>
	pub ForImplicit()() -> Sync

	for i in [1, 2, 3, 4, 5i32] {
	 !"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForImplicitSelector()() -> Sync

	for i: u8 in 1..=5 {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForImplicitItemType()() -> Sync

	for i keyed i => u8 in 1..=5 {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForKeyTypeOnly()() -> Sync

	for i => u8 in &[1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForExplicit()() -> Sync

	for i: u8 keyed i => u8 in [1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForUntyped()() -> Sync

	for i keyed i in [1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
  pub Split()() -> Sync

  for c in "This is a test.".split(' ') {[
	  <li
		!"{:?}"(c)
	  > "\n"
  ]}
}

asteracea::component! { substrate =>
  pub Child()() -> Sync

  for _ in 0..5 {
	  <*ForImplicit>
  }
}
