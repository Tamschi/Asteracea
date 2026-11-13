#[path = "./_sub.rs"]
mod sub;

asteracea::components! {
	#![substrate = sub::Sub]

	pub PlainText => "Testing…";
	pub FormattedText => "Testing…{}{}{}"[1, 2, 3];
}
