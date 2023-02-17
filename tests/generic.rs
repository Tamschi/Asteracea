use asteracea::substrates::web as substrate;
use std::{fmt::Display, marker::PhantomData};

asteracea::component! { substrate =>
	Generic<T: Display>(
		priv _phantom: PhantomData<T> = Default::default(),
	)(
		displayed: T,
	)

	!(displayed)
}

asteracea::component! { substrate =>
	pub User()()

	[
		<*Generic::<i32> .displayed = {0}>
		<*Generic::<u32> .displayed = {1}>
	]
}
