mod _sub;

use std::{marker::PhantomPinned, pin::Pin};

struct Pinned {
	_phantom: PhantomPinned,
}

impl Pinned {
	fn new() -> Self {
		Self {
			_phantom: PhantomPinned,
		}
	}

	fn when_pinned(self: Pin<&Self>) {
		let _ = self;
	}

	fn when_pinned_mut(self: Pin<&mut Self>) {}
}

asteracea::components! {
	#![substrate = _sub::Sub]

	pub Test => {
		let a: Pinned = pin!(Pinned::new());
		let mut b: Pinned = pin!(Pinned::new());

		let cell: i32 = cell!(0);
		(let) cell_ = cell.to_owned();
		let computed: i32 = computed!(move || cell_.get());

		({
			// In `init`.
			a.when_pinned();
			b.as_ref().when_pinned();
			b.as_mut().when_pinned_mut();

			cell.set_if_distinct(1);
			let _ = computed;
		})
		[{
			// In `render`.
			a.when_pinned();
			b.as_ref().when_pinned();
			b.as_mut().when_pinned_mut();
		}]
		"{}, {}"[cell.get(), computed.get()];
	}
}
