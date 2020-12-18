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

<!-- TODO: Explain how matching on components, for example a router, works. Router::INDEX = "\0" -->
