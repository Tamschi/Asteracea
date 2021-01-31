use asteracea::component;
use std::{fmt::Display, marker::PhantomData};

component! {
	Generic<T: Display>(
		priv _phantom: PhantomData<T> = Default::default(),
	)(
		displayed: T,
	)

	!{displayed}
}

component! {
	pub User()()

	[
		<*Generic::<i32> .displayed = {0}>
		<*Generic::<u32> .displayed = {1}>
	]
}
