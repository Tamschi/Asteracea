# `spread if …… else <…>`

You can explicitly specify alternative content with an `else` branch:

```rust asteracea=Alternated
use asteracea::substrates::web;

asteracea::component! { web =>
  Alternates()(
    show_alternative: bool = false,
  )

  spread if {show_alternative}
    "Default"
  else
    "Alternative"
}

asteracea::component! { web =>
  pub Alternated()()

  [
    <*Alternates> "\n"
    <*Alternates .show_alternative = {true}>
  ]
}
```
