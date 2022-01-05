# Asteracea Changelog

<!-- markdownlint-disable no-trailing-punctuation -->

## next

TODO: Date

* **Breaking:**
  * Increased minimum supported Rust version from 1.45.0 to 1.57.0
    > since `asteracea::error` uses `std::panic::panic_any(…)`,
    > and to show better event binding validation errors using const panics.
  * Removed "rhizome" features (always enabled now)
  * Removed "styles" and "topiary" features. CSS scoping will be enabled through more general means.
  * Reworked generated component interface
  * Upgraded `lignin` and `lignin-schema` dependencies to 0.0.3 each
  * Removed all type-level (static) storage declarations. This reduces complexity a lot. Use plain Rust `static` items and, where needed, `new with { …; }` blocks and/or `with { …; } <…>` expressions instead.
  * Reorganised reexported dependencies and made some of them private
  * Upgraded `lignin` dependency from 0.0.3 to 0.0.5
    > to support fallible rendering.
  * Generated `.render(…)` methods are now fallible, returning `Result<_, Escalation>`.
  * The `"backtrace"` feature was replaced by `"tracing"`, which enables [`tracing`](https://docs.rs/tracing/0.1/) compatibility.

* Features:
  * You can now prefix constructor arguments with an explicit visibility (`priv`, `pub`, `pub(restriction)`) to capture them as component instance fields.
  * `bump` resolution is now more reliable in cases where the macro input is constructed in multiple macro contexts.
  * Allow restating element names when closing them (e.g. `/div>`)
  * Added support for unvalidated (custom) HTML element names: `<"custom-element">`
  * HTML comments with `<!-- "comment text" -->`
  * Custom (Asteracea component) child elements:

    ```rust
    <*Name
      *new_arg = {}
      .render_arg = {}
    >
    ```

  * Child component instancing:

    ```rust
    <*Name priv name> // Alternatively: `pub`, `pub(…)`
    <*{self.name_pinned()}> // Without `*new_arg`s!
    ```

  * Optional arguments: `pattern?: Type`
  * Default parameters: `pattern: Type = default`
  * Conditional attributes: `."attribute-name"? = {Option<&'bump str>}`
  * Conditional parameters (like conditional attributes)
  * Boolean attributes: `."attribute-name"? = {bool}`
  * `new with { …; }` blocks to insert statements into the constructor
  * `with { …; } <…>` expressions to insert statements into the `.render` method
  * Conditional content via `if {…} <…>`, `if …… else <…>` and `match <…> [ … ]`
  * Box expressions: `box ⟦priv …⟦: ⟦struct⟧ … ⟦where …;⟧⟧⟧ <…>`

* Revisions:
  * Improved `Counter` example in the README.

## 0.0.2

2020-10-08

* Features:
  * Added shorthand formatting syntax:

    Use `!{value}` to format plainly using `Display`.  
    Use e.g. `!"{} {}"{value1, value2}` to specify a custom format string.

## 0.0.1

2020-10-05

Initial unstable release
