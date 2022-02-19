#![allow(clippy::reversed_empty_ranges)]

asteracea::component! {
	Empty()()

	*for _ in 0..0 {
		[]
	}
}
