# `match <…> [ … ]`

Rust's `match` statements are available in Asteracea contexts, with slightly changed syntax:

```rust asteracea=Matched
enum Enum<'a> {
  Text(&'a str),
  Other,
}

asteracea::component! {
  MatchEnum()(
    enum_value: Enum<'_>,
  )

  match {enum_value} [
    Enum::Text(text) => <span !{text}>
    Enum::Other => <div ."class" = "placeholder">
  ]
}

asteracea::component! {
  pub Matched()()

  [
    <*MatchEnum .enum_value = { Enum::Text("Hello!") }> "\n"
    <*MatchEnum .enum_value = { Enum::Other }>
  ]
}
```

Note that the match expression accepts an element expression (`<…>`) as parameter! If an element renders into something other than a [`Node`](), you can branch on that result this way:

```rust asteracea=RouterUser
asteracea::component!{
  Router()() -> &'_ str

  //TODO: Retrieve from fragment.
  { "\0" }
}

impl Router {
  const INDEX: &'static str = "\0";
}

asteracea::component! {
  RouterUser()()

  match <*Router> [
    Router::INDEX | "" => "Index"
    _ => {unreachable!()}
  ]
}
```
