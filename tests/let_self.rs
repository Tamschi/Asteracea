struct S;
impl S {
	fn new() -> Self {
		S
	}

	fn s(&self) -> &str {
		"s"
	}
}

asteracea::component! {
	pub LetSelf(
		a: u32,
		b: u32,
	)() -> Sync

	let self.a: u32 = a;
	let self.b: u32 = pin b;
	let self.s = S::new();

	!"{}{}{}"(self.a, self.b_pinned(), self.s.s())
}

asteracea::component! {
  pub Attributed()() -> Sync

  // TODO: Also support outer attributes again.
  let self.a: u8 = #![allow(dead_code)] 0;

  "Hello!"
}
