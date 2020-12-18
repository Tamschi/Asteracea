pub enum Enum<'a> {
	Text(&'a str),
	Other,
}

asteracea::component! {
	pub MatchEnum()(
		enum_value: Enum<'_>,
	)

	match {enum_value} [
		Enum::Text(text) => <span !{text}>
		Enum::Other => <div ."class" = "placeholder">
	]
}
