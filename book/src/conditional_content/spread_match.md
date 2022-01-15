# `spread match { … } [ … ]`

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

  spread match {enum_value} [
    Enum::Text(text) => <span !(text)>
    Enum::Other => <div ."class" = "placeholder">
  ]
}

asteracea::component! {
  pub Matched()() -> Sync

  [
    <*MatchEnum .enum_value = { Enum::Text("Hello!") }> "\n"
    <*MatchEnum .enum_value = { Enum::Other }>
  ]
}
```
