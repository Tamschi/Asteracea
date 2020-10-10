use asteracea::component;

macro_rules! just_plain {
	($name:ident => $($contents:tt)*) => {
		component! {
			$name()()
			$($contents)*
		}
	}
}

macro_rules! nested {
	($name:ident => $($contents:tt)*) => {
		just_plain!($name => $($contents)*);
	}
}

just_plain!(JustPlain => <br>);
nested!(Nested => <br>);
