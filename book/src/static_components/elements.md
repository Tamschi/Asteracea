# Elements

To define elements and their contents, Asteracea uses a syntax similar to HTML:

```rust asteracea=Div::new()
asteracea::component! {
  pub Div()()

  <div>
}
```

`<name` opens an element and `>` is enough to close one. However, you can alternatively close elements with `/name>` too, which the compiler will validate:

```rust asteracea=Div::new()
asteracea::component! {
  pub Div()()

  <div
    // [complex nested template]
  /div>
}
```

Elements can contain text:

```rust asteracea=Span::new()
asteracea::component! {
  pub Span()()

  <span "This is text within a <span>.">
}
```

Elements can be nested:

```rust asteracea=DivSpan::new()
asteracea::component! {
  pub DivSpan()()

  <div
    <span "This is text within a <span>.">
  >
}
```

Element names are statically validated against [`lignin-schema`] by default, but this can be amended by importing additional schema functions. <!-- TODO: Example! -->

[`lignin-schema`]: TK

<!-- To use custom element names, quote them like this:

```rust asteracea=Custom::new()
asteracea::component! {
  pub Custom()()

  <"custom-element">
}
```
-->
