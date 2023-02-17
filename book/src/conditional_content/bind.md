# bind ⦃storage⦄ ⟦move⟧ <…>

Much like `defer` expressions, `bind` expressions evaluate a sub-expression constructor once when first rendered.

However, they also *shift the constructor scope of a sub-expression into its parent's render scope*.

This lets you use render parameters as constructor parameters:

```rust asteracea=Test
use asteracea::substrates::web;

asteracea::component! { web =>
  Early(
    priv early: &'static str,
  )()

  ["Constructor parameter: " !"{:?}"(self.early)]
}

asteracea::component! { web =>
  Late()(
    late: &'static str,
  )

  bind <*Early *early = {late}>
}

asteracea::component! { web =>
  pub Test()()

  [
    <*Late priv tested .late = {"first"}> "\n"
    <*{self.tested_pinned()} .late = {"second"}>
  ]
}
```

As you can see, `early` is only assigned once. `late` is discarded during the second call.

By default, the sub-expression constructor acts like (read: is) a plain Rust closure without additional keywords, but you can apply the `move` keyword after the optional storage configuration.
