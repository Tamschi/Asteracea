mod old {
	#![allow(dead_code, deprecated, clippy::reversed_empty_ranges)]
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
}

asteracea::components! {
	Empty -> web {
		(for) _ in 0..0 {}
	}

	WithinHtml -> web {
		div {
			(for) _ in 0..0 {}
		}
	}

	Container[content: '_] -> web {
		..content;
	}


	AsContent -> web {
		Container {
			(for) _ in 0..0 {}
		}
	}
}
