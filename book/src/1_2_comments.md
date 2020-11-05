# Chapter 1.2: Comments

You can use three distinct types of comments in Asteracea macros, all serving different purposes:

First, standard Rust comments can be placed anywhere in Asteracea components (or any other place in a Rust program), and are not included in the compiled binary:

```rust asteracea=Commented::new()
asteracea::component! {
  pub Commented()()

  [
    // This is a one-line comment.
    /*
    /* These are *nested* multiline comments. */
    */
  ]
}
```

Additionally, Rust documentation is supported in many places:

```rust asteracea=Documented::new()
asteracea::component! {
  /// This is a documented component.  
  /// Running `cargo doc` will pick up on its documentation.
  pub Documented()()

  []
}
```

These `///` (or `//!`) annotations are not included in the compiled binary either¹, but can be picked up by standard Rust tooling like [rust-analyzer].

¹ Rare exceptions in combination with other macros apply.

The third kind of comment is specific to Asteracea and does affect program output:

```rust asteracea=HtmlCommented::new() asteracea::render=.render()
asteracea::component! {
  pub HtmlCommented()()

  <!-- "This is an HTML comment." -->
}
```

The double quotes are a Rust limitation: Since Rust tokenises macro input, a string literal is required to extract raw text.

You can use a multiline string to easily write a multiline HTML comment:

```rust asteracea=HtmlCommented::new() asteracea::render=.render()
asteracea::component! {
  pub HtmlCommented()()

  <!-- "
    This comment spans mul-
    tiple lines, I hope it is
    not too annoying.
  " -->
}
```

[rust-analyzer]: https://rust-analyzer.github.io/
