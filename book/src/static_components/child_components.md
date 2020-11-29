# Child Components

Asteracea components can be used inside other templates using asterisk syntax:

```rust asteracea=Outer
asteracea::component! {
  Inner()()
  "Inner body."
}

asteracea::component! {
  Outer()()
  <*Inner>
}
```

Using a component multiple times results in distinct instances:

```rust asteracea=Outer
//TODO: Hide this initially.
asteracea::component! {
  Inner()()
  "Inner body."
}

asteracea::component! {
  Outer()()
  [
    <*Inner>
    <*Inner>
  ]
}
```

## Child Component Instancing

> Note: Rust is good at erasing empty instances!
>
> If your reused component is stateless, please restate the component's type name instead of using instancing. This will keep your code clearer and less interdependent.
>
> For more information, see [The Rustonomicon on Zero Sized Types (ZSTs)](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts).

Instead of instantiating and storing a child component multiple times, you can instance it by giving it a name and referencing it elsewhere through a Rust block:

```rust asteracea=Outer
//TODO: Hide this initially.
asteracea::component! {
  Inner()()
  "Inner body."
}

asteracea::component! {
  Outer()()
  [
    <*Inner priv inner> // Alternatively: `pub` or `pub(…)`
    <*{&self.inner}>
  ]
}
```

The component's `.render(…)` method is called for each of these appearances, but `::new(…)` is called only once.

Component instancing is especially useful when rendering alternates, since the child instance is available everywhere in the parent component's body (regardless which `.render(…)` path is taken).
