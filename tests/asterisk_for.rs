#![allow(clippy::reversed_empty_ranges)]

asteracea::component! {
	Empty()()

	*for _ in 0..0 {
		[]
	}
}

asteracea::component! {
	WithinHtml()()

	<div
		*for _ in 0..0 {
			[]
		}
	>
}

asteracea::component! {
	Container()(..)

	..
}

asteracea::component! {
	AsContent()()

	<*Container
		*for _ in 0..0 {
			[]
		}
	>
}
