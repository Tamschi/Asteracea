use asteracea::substrates::web as substrate;

asteracea::component! { substrate =>
	pub ForImplicit()()

	for i in [1, 2, 3, 4, 5i32] {
	 !"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForImplicitSelector()()

	for i: u8 in 1..=5 {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForImplicitItemType()()

	for i keyed i => u8 in 1..=5 {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForKeyTypeOnly()()

	for i => u8 in &[1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForExplicit()()

	for i: u8 keyed i => u8 in [1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
	pub ForUntyped()()

	for i keyed i in [1, 2, 3, 4, 5] {
		!"{}"(i)
	}
}

asteracea::component! { substrate =>
  pub Split()()

  for c in "This is a test.".split(' ') {[
	  <li
		!"{:?}"(c)
	  > "\n"
  ]}
}

asteracea::component! { substrate =>
  pub Child()()

  for _ in 0..5 {
	  <*ForImplicit>
  }
}
