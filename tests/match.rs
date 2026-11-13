mod _sub;

pub enum Enum<'a> {
	Text(&'a str),
	Other,
}

asteracea::components! {
	#![substrate = _sub::Sub]

	pub MatchEnum[
		enum_value: Enum<'_>,
	] => [match] enum_value {
		Enum::Text(text) => <span !(text)>
		Enum::Other => <div .class = "placeholder">
	}
}
