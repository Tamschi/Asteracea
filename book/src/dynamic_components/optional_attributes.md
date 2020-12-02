# Optional Attributes

Asteracea supports boolean and otherwise optional attributes with the following syntax:

```rust asteracea=Classical
asteracea::component! {
  Classic()(
    // This will be improved on in the next chapters.
    class: Option<&'bump str>,
  )

  <div
    ."class"? = {class}
  >
}

asteracea::component! {
  Classical()()

  [
    <*Classic .class = {None}> "\n"
    <*Classic .class = {Some("classicist")}>
  ]
}
```

Instead of [`&'bump str`](), the required value type is [`Option<&'bump str>`]().

If [`None`]() is provided, the attribute is omitted entirely from the rendered VDOM. To render a [boolean attribute](https://www.w3.org/TR/html52/infrastructure.html#sec-boolean-attributes) like `checked`, provide [`Some("")`]().
