use std::mem::size_of;

#[test]
fn stateless_components_are_zero_sized() {
	asteracea::component! { substrate =>
		Empty()()
		""
	}

	assert_eq!(size_of::<Empty>(), 0)
}
