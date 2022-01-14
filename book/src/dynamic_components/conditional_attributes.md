# Conditional Attributes

Asteracea supports conditionally setting optional attributes with the following syntax:

```rust asteracea=Classical
asteracea::component! {
  Classic()(
    // This will be improved on in the next chapters.
    class: Option<&'bump str>,
  )

  <div
    .class? = {class}
  >
}

asteracea::component! {
  Classical()()

  [
    <*Classic .class = {None}> "\n"
    <*Classic .class = {Some("classicist")}>
  ]
}
```

Instead of [`&'bump str`](), the attribute value type here is [`Option<&'bump str>`](). If [`None`]() is provided, the attribute is omitted entirely from the rendered VDOM.

This can be used to conditionally render a [boolean attribute](https://www.w3.org/TR/html52/infrastructure.html#sec-boolean-attributes) like `checked`, providing [`Some("")`]() to enable the attribute. However, it is usually more convenient to use a [`bool`]() directly:

## Boolean Attributes

To make dynamic boolean attributes like [`hidden`](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/hidden) more convenient to use, conditional attributes also accept [`bool`]() values directly:

```rust asteracea=Outer
asteracea::component! {
  Vis()(
    visible: bool,
  )

  <div
    .hidden? = {!visible}
    "#"
  >
}

asteracea::component! {
  Outer()()

  [
    <*Vis .visible = {true}> "\n"
    <*Vis .visible = {false}>
  ]
}
```

[`true`]() is converted to [`Some("")`]() and [`false`]() to [`None`]() in this case, [as per specification](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#boolean-attributes).

> Which types are compatible with conditional attributes is controlled by the [`ConditionalAttributeValue`]() trait.
>
> It is by default implemented for `bool` and `Option<&'bump str>`, and I recommend **not** extending this list unless the conversion is very fast.
