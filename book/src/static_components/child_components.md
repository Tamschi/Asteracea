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
    <*Inner priv inner> // Alternatively: `pub` etc.
    <*{&self.inner}>
  ]
}
```
