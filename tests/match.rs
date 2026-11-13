mod _sub;

pub enum Enum<'a> {
	Text(&'a str),
	Other,
}

asteracea::components! {
	#![substrate = _sub::Sub]

	pub MatchEnum[
		enum_value: Enum<'_>,
	] => [match] todo!("enum_value") {
		Enum::Text(text) if true => span{ "{}"[text]; },
		Enum::Text(text) => [{ unreachable!() }],
		Enum::Other => div[class = "placeholder"];,
	}
}
