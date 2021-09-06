# bind ⦃storage⦄ ⟦move⟧ <…>

Much like `defer` expressions, `bind` expressions evaluate a sub-expression constructor once when first rendered.

However, they also *shift the constructor scope of a sub-expression into its parent's render scope*.

This lets you use render parameters as constructor parameters:

```rust asteracea=Test
asteracea::component! {
  Early(
    priv early: &'static str,
  )()

  ["Constructor parameter: " !"{:?}"{self.early}]
}

asteracea::component! {
  Late()(
    late: &'static str,
  )

  bind <*Early *early = {late}>
}

asteracea::component! {
  pub Test()() -> Sync?

  [
    <*Late priv tested .late = {"first"}> "\n"
    <*{self.tested_pinned()} .late = {"second"}>
  ]
}
```

As you can see, `early` is only assigned once. `late` is discarded during the second call.

By default, the sub-expression constructor acts like (read: is) a plain Rust closure without additional keywords, but you can apply the `move` keyword after the optional storage configuration.
