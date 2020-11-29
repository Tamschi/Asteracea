# Asteracea Changelog

## next

TODO: Date

* **Breaking:**
  * Increased minimum supported Rust version from 1.45.0 to 1.46.0.
  * Removed "rhizome" features (always enabled now)
  * Removed "styles" and "topiary" features. CSS scoping will be enabled through more general means.
  * Reworked generated component interface
  * Upgraded `lignin` dependency to 0.0.2
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
