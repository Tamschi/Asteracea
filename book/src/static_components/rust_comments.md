# Rust Comments

You can use three distinct types of comments in Asteracea macros, all serving different purposes:

First, standard Rust comments can be placed anywhere in Asteracea components (or any other place in a Rust program), and are not included in the compiled binary:

```rust asteracea=Commented
asteracea::component! { substrate =>
  Commented()()

  [
    // This is a one-line comment.
    /*
    /* These are *nested* multiline comments. */
    */
  ]
}
```

Additionally, Rust documentation is supported in many places:

```rust asteracea=Documented
asteracea::component! { substrate =>
  /// This is a documented component.  
  /// Running `cargo doc` will pick up on its documentation.
  pub Documented()() -> Sync

  []
}
```

These `///` (or `//!`) annotations are not included in the compiled binary either¹, but can be picked up by standard Rust tooling like [rust-analyzer].

¹ Rare exceptions in combination with other macros apply.

[rust-analyzer]: https://rust-analyzer.github.io/
