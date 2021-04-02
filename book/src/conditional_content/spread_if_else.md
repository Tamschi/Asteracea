# `spread if …… else <…>`

You can explicitly specify alternative content with an `else` branch:

```rust asteracea=Alternated
enum Which {
  First,
  Second,
}

asteracea::component! {
  Alternates()(
    show_alternative: bool = false,
  )

  spread if {show_alternative}
    "Default"
  else
    "Alternative"
}

asteracea::component! {
  pub Alternated()() -> Sync

  [
    <*Alternates> "\n"
    <*Alternates .show_alternative = {true}>
  ]
}
```
