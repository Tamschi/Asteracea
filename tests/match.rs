pub enum Enum<'a> {
	Text(&'a str),
	Other,
}

asteracea::component! { substrate =>
	pub MatchEnum()(
		enum_value: Enum<'_>,
	) -> Sync

	spread match {enum_value} [
		Enum::Text(text) => <span !(text)>
		Enum::Other => <div .class = "placeholder">
	]
}
