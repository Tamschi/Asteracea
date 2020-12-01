# Optional Attributes

Asteracea supports boolean and otherwise optional attributes with the following syntax:

```rust asteracea=Classical
asteracea::component! {
  Classic()(
    class: Option<&'bump str>, //TODO: Use optional argument once available.
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

Instead of [`&'bump str`], the required value type is [`Option<&'bump str>`].

If [`None`] is provided, the attribute is omitted entirely from the rendered VDOM.
