# `GUIError` and `"backtrace"`

The `::new(…)` and `.render(…)` functions of Asteracea-components are fallible, returning a `Result<_, GUIError>`.

[`GUIError`]()s are panic-like: They are not expected during normal execution of a web frontend, are strictly deprioritised compared to the [`Ok`]() path and components that catch them are encouraged to implement a "fail once and stop" approach where child components are disposed of on first failure.

As long as Asteracea is compiled with the `"backtrace"` feature, it will trace [`GUIError`]() propagation through any function instrumented via the [`#[asteracea::gui_tracing]`]() attribute, which is automatic for the two mentioned above.

You can escalate any error along the GUI tree as long as it is `Any`, `Error` and `Send`.

```rust asteracea=Outer
use asteracea::error::IntoGUIResult;
use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
struct AnError;
impl Display for AnError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    writeln!(f, "A test error was raised")
  }
}
impl Error for AnError {}

asteracea::component! {
  Failing()()

  {
    // Raising a `GUIError` means crashing at least part of the app,
    // so there is a speed bump for this conversion.
    return Err(AnError).into_gui_result();
  }
}

asteracea::component! {
  Containing()()

  <*Failing>
}

asteracea::component! {
  pub Outer()()

  <*Containing>
}
```

These backtraces are for human consumption, so please don't parse them. They may change in any release without notice.

> If the `"force-unwind"` feature is enabled, `GUIError`s are erased and the type itself uses the panic infrastructure for propagation instead of being passed up via [`Err`]() variant. This may reduce code size in some cases.
>
> However, note that **panics cannot be caught on platforms without unwinding, including Wasm** (as of Rust 1.49.0).
>
> In the future, panic conversion will be activated automatically on compatible platforms, as long as this can be done without compromising backtraces.
