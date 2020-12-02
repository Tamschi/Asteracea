# Optional Arguments

When working with values that may or may not be provided, where the default is outside the range of possible provided values, you can improve your component's interface towards consumers by using optional arguments:

```rust asteracea=Classical
asteracea::component! {
  Classic()(
    class?: &'bump str,
  )

  <div
    ."class"? = {class} // `Option<_>`-typed!
  >
}

asteracea::component! {
  Classical()()

  [
    <*Classic> "\n"
    <*Classic .class = {"classicist"}> // Not `Option<_>`-typed!
  ]
}
```

`class` is an `Option<&'bump str>` within `Classic`s `.render(…)` method, but the parameter is provided from outside as `&'bump str`.

## Conditional Child Component Parameters

Note that this means that `None` can only be specfied by not setting the parameter at all! Fortunately, it's easy to do this conditionally in the same way as for optional attributes on HTML elements:

```rust asteracea=Outer
asteracea::component! {
  Inner()(
    class?: &'bump str
  )

  <span ."class"? = {class}>
}

asteracea::component! {
  Middle()(
    class?: &'bump str
  )

  <*Innter .class? = {class}>
}

asteracea::component! {
  Outer()()

  [
    <*Middle> "\n"
    <*Middle .class = {"bourgeoisie"}>
  ]
}
```

This also applies to any other kind of optional parameter, i.e. arguments with explicit default value.

> Asteracea assumes that the order of distinctly named parameter assignments does not matter and, in order to reduce code size, internally moves assignments of parameters that are assigned to conditionally after those of ones that are not. This is a tradeoff incurred by statically validating parameter list completeness.
>
> **Parameter value evaluation order**, assignment order among any unconditionally set parameters, assignment order among any optionally set parameters **and the order of assignments to the same name are preserved regardless**.
>
> As long as your program compiles, this optimisation is unobservable when handling child component types created using `asteracea::component! { … }`. You may still want to keep it in mind for components with custom argument builders.
