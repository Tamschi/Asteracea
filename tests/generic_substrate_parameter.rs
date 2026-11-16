use asteracea::{components, substrate::bounds::SimpleContentSubstrate};

components! {
	#![substrate = S where S: SimpleContentSubstrate<'r, &'static str>]

	pub Hello => "Hello, world!";
}
