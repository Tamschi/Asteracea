# Elements

To define elements and their contents, Asteracea provides a syntax similar to HTML:

```rust asteracea=Div
use asteracea::substrates::web;

asteracea::component! { web =>
  Div()()

  <div>
}
```

`<name` opens an element and `>` is enough to close one. However, you can alternatively close elements with `/name>` too, which the compiler will validate:

```rust asteracea=Div
use asteracea::substrates::web;

asteracea::component! { web =>
  Div()()

  <div
    // [complex nested template]
  /div>
}
```

Elements can contain any number of valid Asteracea component bodies, which are rendered as the element's children, as long as the specific element supports it:

```rust asteracea=Span
use asteracea::substrates::web;

asteracea::component! { web =>
  Span()()

  <span
    "This is text within a <span>."
    <!-- "This is a comment within a <span>." -->
  >
}
```

This includes other elements:

```rust asteracea=DivSpan
use asteracea::substrates::web;

asteracea::component! { web =>
  DivSpan()()

  <div
    <span "This is text within a <span>.">
  >
}
```

Elements are statically validated against [`lignin-schema`].

[Empty elements] like `<br>` are written like any other element, but don't accept children and won't render a closing tag to HTML when using [lignin-html]:

[Empty elements]: https://developer.mozilla.org/en-US/docs/Glossary/empty_element
[lignin-html]: TK

```rust asteracea=Br
use asteracea::substrates::web;

asteracea::component! { web =>
  Br()()

  <br>
}
```

[`lignin-schema`]: TK

To use custom element names without validation, quote them like this:

```rust asteracea=Custom
use asteracea::substrates::web;

asteracea::component! { web =>
  Custom()()

  <"custom-element">
}
```
