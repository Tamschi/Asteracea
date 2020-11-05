# Chapter 1.3: HTML comments

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
