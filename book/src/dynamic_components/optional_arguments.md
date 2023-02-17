# Optional Arguments

When working with values that may or may not be provided, where the default is outside the range of possible provided values, you can improve your component's interface towards consumers by using optional arguments:

```rust asteracea=Classical
asteracea::component! { substrate =>
  Classic()(
    class?: &'bump str,
  )

  <div
    .class? = {class} // `Option<_>`-typed!
  >
}

asteracea::component! { substrate =>
  Classical()()

  [
    <*Classic> "\n"
    <*Classic .class = {"classicist"}> // Not `Option<_>`-typed!
  ]
}
```

`class` is an `Option<&'bump str>` within `Classic`s `.render(…)` method, but the parameter is provided from outside as `&'bump str`.
