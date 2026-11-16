use std::fmt::Arguments;

use asteracea::{components, substrate::bounds::Projects};

components! {
	#![substrate = S
		where S: Projects<&'static str>
			+ Projects<Arguments<'r>>
	]

	pub Hello => "Hello, world!";
}
