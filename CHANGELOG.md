# Asteracea Changelog

## next

* **Breaking:**

  * Increased minimum supported Rust version from 1.45.0 to 1.46.0.

* The bump format shorthand now does not imply `'a: 'bump` anymore, where `'a` is the component's lifetime.

* Improved `Counter` example in the README.

* You can now prefix constructor arguments with an explicit visibility (`priv`, `pub`, `pub(restriction)`) to cature them as component instance fields.

## 0.0.2

2020-10-08

* Added shorthand formatting syntax:

  * Use `!{value}` to format plainly using `Display`.

  * Use e.g. `!"{} {}"{value1, value2}` to specify a custom format string.

## 0.0.1

2020-10-05

Initial unstable release
