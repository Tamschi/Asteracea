# Boolean Attributes

Asteracea supports boolean and otherwise optional attributes with the following syntax:

```rust asteracea=Classic
asteracea::component! {
  Classic()(
    class: Option<&'bump str>, //TODO: Use optional argument once available.
  )

  <div
    ."class"? = {class}
  >
}
```

Instead of [`&'bump str`], the required value type is [`Option<&'bump str>`].

If [`None`] is provided, the attribute is omitted entirely from the rendered VDOM.
