# Argument Defaults

Asteracea provides multiple ways to make working with optional arguments easier.

Like in for example TypeScript, you can specify default parameters for constructor and render arguments:

```rust asteracea=Classical
asteracea::component! { substrate =>
  Classic()(
    // This will be improved on in the next chapter.
    class: Option<&'bump str> = None,
  )

  <div
    .class? = {class}
  >
}

asteracea::component! { substrate =>
  Classical()()

  [
    <*Classic> "\n" // Parameter omitted.
    <*Classic .class = {Some("classicist")}>
  ]
}
```

Default parameter expressions are normal Rust expressions, and are evaluated as needed if the parameter was not specified.

<!-- TODO: Figure out if default parameter expressions can see other parameters, if yes which, and then clarify that here. -->