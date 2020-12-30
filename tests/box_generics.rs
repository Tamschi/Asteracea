use std::fmt::Debug;

asteracea::component! {
	Boxed<T>()()

	box <*Boxed::<T>>
}

asteracea::component! {
	Boxedd<S, T>()()

	[
		box <*Boxed::<S>>
		box <*Boxed::<T>>
	]
}

struct Predefined<T> {
	boxed: Boxed<T>,
}

asteracea::component! {
	Predefinedd<S, T>()()

	[
		box priv a: Predefined::<S> <*Boxed::<S> priv boxed>
		box priv b: Predefined::<T> <*Boxed::<T> priv boxed>
	]
}

asteracea::component! {
	Claused<T: Debug>()()

	box <*Claused::<T>>
}

asteracea::component! {
	Whered<T> where T: Debug, ()()

	box <*Whered::<T>>
}

asteracea::component! {
	Custom<T>()()

	box priv a: struct C::<T> <*Custom::<T>>
}

asteracea::component! {
	CustomClaused<T: Debug>()()

	box priv a: struct CC::<T: Debug> <*CustomClaused::<T>>
}

asteracea::component! {
	CustomWhered<T> where T: Debug, ()()

	box priv a: struct CW::<T> where T: Debug; <*CustomWhered::<T>>
}
