# Conditional Child Component Parameters

Note that providing optional argument values without [`Some`]() means that [`None`]() can only be specfied by not setting the parameter at all! Fortunately, it's easy to do this conditionally in the same way as for optional attributes on HTML elements:

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

  <*Inner .class? = {class}>
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

> Asteracea constructs child component parameter bundles using a builder pattern. It assumes that the order of distinctly named parameter assignments does not matter and, in order to reduce code size, internally moves assignments of parameters that are assigned to conditionally after those of ones that are not. This is a tradeoff incurred by statically validating parameter list completeness.
>
> **Parameter value evaluation order**, assignment order among any unconditionally set parameters, assignment order among any optionally set parameters **and the order of assignments to the same name are preserved regardless**.
>
> As long as your program compiles, this optimisation is unobservable when handling child component types created using `asteracea::component! { … }`. You may still want to keep it in mind for components with custom argument builders.
