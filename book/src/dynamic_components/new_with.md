# `new with { …; }`

<!-- The reason for not placing this block after the constructor arguments (without keyword) is that this would create a lot of separation between constructor and render arguments, which should both be visible at a glance when peeking at a component's source code. -->

Arbitrary Rust code can be inserted into a component's constructor using a `new with`-block:

```rust asteracea=Constructed
asteracea::component! {
  Constructed()()

  new with {
    // TODO
  }

  []
}
```

Code inside the `new with`-block has access to constructor parameters, and `let`-bindings from this block are in scope for capture initialisers:

```rust asteracea=Quoter
asteracea::component! {
  QuoteHolder(
    text: &str,
  )()

  new with {
    let quoted = format!("‘{}’", text);
  }

  let self.quote: String = quoted;

  !(self.quote)
}

asteracea::component! {
  Quoter()()

  <*QuoteHolder *text = { "This text is quoted." }>
}
```
