use asteracea::substrates::web as substrate;
use std::{fmt::Debug, pin::Pin};

asteracea::component! { substrate =>
	pub Boxed<T>()()

	box <*Boxed::<T>>
}

asteracea::component! { substrate =>
	pub Boxedd<S, T>()()

	[
		box <*Boxed::<S>>
		box <*Boxed::<T>>
	]
}

struct Predefined<T> {
	boxed: Boxed<T>,
}

impl<T> Predefined<T> {
	fn boxed_pinned(self: Pin<&Self>) -> Pin<&Boxed<T>> {
		unsafe { self.map_unchecked(|p| &p.boxed) }
	}
}

asteracea::component! { substrate =>
	pub Predefinedd<S, T>()()

	[
		box priv a: Predefined::<S> <*Boxed::<S> priv boxed>
		box priv b: Predefined::<T> <*Boxed::<T> priv boxed>
	]
}

asteracea::component! { substrate =>
	pub Claused<T: Debug>()()

	box <*Claused::<T>>
}

asteracea::component! { substrate =>
	pub Whered<T> where T: Debug, ()()

	box <*Whered::<T>>
}

asteracea::component! { substrate =>
	#[allow(dead_code)] // Used below; Waiting on min_specialization.
	Picky<T: Debug>()()

	box []
}

// Waiting on min_specialization.
// asteracea::component! { substrate =>
// 	Custom<T>()()

// 	box priv a: struct C::<T> <*Boxed::<T>>
// }

// asteracea::component! { substrate =>
// 	CustomClaused<T: Debug>()()

// 	box priv a: struct CC::<T: Debug> <*Picky::<T>>
// }

// asteracea::component! { substrate =>
// 	CustomWhered<T> where T: Debug, ()()

// 	box priv a: struct CW::<T> where T: Debug; <*Picky::<T>>
// }
