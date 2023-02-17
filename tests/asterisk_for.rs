#![allow(clippy::reversed_empty_ranges)]

asteracea::component! { substrate =>
	Empty()()

	*for _ in 0..0 {
		[]
	}
}

asteracea::component! { substrate =>
	WithinHtml()()

	<div
		*for _ in 0..0 {
			[]
		}
	>
}

asteracea::component! { substrate =>
	Container()(..)

	..
}

asteracea::component! { substrate =>
	AsContent()()

	<*Container
		*for _ in 0..0 {
			[]
		}
	>
}
