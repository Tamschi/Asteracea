# `do {…}`

<!-- The reason for not placing this block after the constructor arguments (without keyword) is that this would create a lot of separation between constructor and render arguments, which should both be visible at a glance when peeking at a component's source code. -->

Arbitrary Rust code can be inserted into a component's constructor using a `do`-block:

```rust asteracea=Constructed
asteracea::component! {
  pub Constructed()()

  do {
    // TODO
  }

  []
}
```

Code inside the `do`-block has access to constructor parameters, and `let`-bindings from the constructor block are in scope for capture initialisers:

```rust asteracea=QuoteHolder
asteracea::component! {
  pub QuoteHolder(
    text: &str,
  )()

  do {
    let quoted = format!("‘{}’", text);
  }

  //TODO:
  // Captures should be legal for all dynamic value expressions…
  // This code will turn into `!|quote: String = { quoted }|` then.
  |quote: String = { quoted }|;

  !{self.quote}
}
```
