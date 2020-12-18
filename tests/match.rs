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

asteracea::component! {
  Router()() -> &'_ str

  //TODO: Retrieve from fragment.

  { "\0" }
}

impl Router {
	const INDEX: &'static str = "\0";
}

asteracea::component! {
  pub RouterUser()()

  match <*Router> [
	Router::INDEX | "" => "Index"
	_ => {unreachable!()}
  ]
}
