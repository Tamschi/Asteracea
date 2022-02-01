# `for`-loops

`for` loops in Asteracea components resemble those in plain Rust, but do produce output on each iteration:

```rust asteracea=Looped
asteracea::component! {
  pub Looped()() -> Sync

  for c: &str in "This is a test.".split(' ') {[
      <li
        !"{:?}"(c)
      > "\n"
  ]}
}
```
