# `spread match { … } [ … ]`

Rust's `match` statements are available in Asteracea contexts, with slightly changed syntax:

```rust asteracea=Matched
use asteracea::substrates::web;

enum Enum<'a> {
  Text(&'a str),
  Other,
}

asteracea::component! { web =>
  MatchEnum()(
    enum_value: Enum<'_>,
  )

  spread match {enum_value} [
    Enum::Text(text) => <span !(text)>
    Enum::Other => <div ."class" = "placeholder">
  ]
}

asteracea::component! { web =>
  pub Matched()()

  [
    <*MatchEnum .enum_value = { Enum::Text("Hello!") }> "\n"
    <*MatchEnum .enum_value = { Enum::Other }>
  ]
}
```
