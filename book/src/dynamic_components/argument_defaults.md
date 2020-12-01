# Argument Defaults

Like in for example TypeScript, you can specify defaults parameters for constructor and render arguments:

```rust asteracea=Greeting
asteracea::component! {
  Greeting()(
    greeting: &str = "Hello!",
  )

  <span
    ."class" = "greeting"
    !{greeting}
  >
}
```

Default parameter expressions are normal Rust expressions, and are evaluated as needed if the parameter was not specified.

<!-- TODO: Figure out if default parameter expressions can see other parameters, if yes which, and then clarify that here. -->