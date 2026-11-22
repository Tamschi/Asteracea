use std::fmt::Arguments;

use asteracea::{components, substrate::bounds::Projects};

components! {
	#![substrate = S
		//TODO: Should this require `IsSubstrate`?
		where S: 'static
			+ Projects<&'static str>
			+ Projects<Arguments<'r>>
	]

	pub Hello => "Hello, world!";
}
