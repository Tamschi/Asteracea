# Elements

To define elements and their contents, Asteracea provides a syntax similar to HTML:

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

Elements can contain any number of valid Asteracea component bodies, which are rendered as the element's children, as long as the specific element supports it:

```rust asteracea=Span::new()
asteracea::component! {
  pub Span()()

  <span
    "This is text within a <span>."
    <!-- "This is a comment within a <span>." -->
  >
}
```

This includes other elements:

```rust asteracea=DivSpan::new()
asteracea::component! {
  pub DivSpan()()

  <div
    <span "This is text within a <span>.">
  >
}
```

Element names are statically validated against [`lignin-schema`] by default, but this can be amended by importing additional schema functions. <!-- TODO: Example! -->

[Empty elements] like `<br>` are written like any other element but don't accept children and won't render a closing tag to HTML when using [lignin-html]:

[Empty elements]: https://developer.mozilla.org/en-US/docs/Glossary/empty_element
[lignin-html]: TK

```rust asteracea=Br::new()
asteracea::component! {
  pub Br()()

  <br>
}
```

[`lignin-schema`]: TK

To use custom element names without validation, quote them like this:

```rust asteracea=Custom::new()
asteracea::component! {
  pub Custom()()

  <"custom-element">
}
```
