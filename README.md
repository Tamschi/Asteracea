# Asteracea

[![Lib.rs](https://img.shields.io/badge/Lib.rs-*-84f)](https://lib.rs/crates/asteracea)
[![Crates.io](https://img.shields.io/crates/v/asteracea)](https://crates.io/crates/asteracea)
[![Docs.rs](https://docs.rs/asteracea/badge.svg)](https://docs.rs/crates/asteracea)

![Rust 1.45.0](https://img.shields.io/static/v1?logo=Rust&label=&message=1.45.0&color=grey)
[![Build Status](https://travis-ci.com/Tamschi/Asteracea.svg?branch=develop)](https://travis-ci.com/Tamschi/Asteracea/branches)
![Crates.io - License](https://img.shields.io/crates/l/asteracea/0.0.2)

[![GitHub](https://img.shields.io/static/v1?logo=GitHub&label=&message=%20&color=grey)](https://github.com/Tamschi/Asteracea)
[![open issues](https://img.shields.io/github/issues-raw/Tamschi/Asteracea)](https://github.com/Tamschi/Asteracea/issues)
[![open pull requests](https://img.shields.io/github/issues-pr-raw/Tamschi/Asteracea)](https://github.com/Tamschi/Asteracea/pulls)
[![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/asteracea.svg)](https://web.crev.dev/rust-reviews/crate/asteracea/)

Asteracea is a web application framework aiming to combine the strengths of [Angular] and [React] while fully supporting Rust's lifetime model.

[Angular]: https://angular.io/
[React]: https://reactjs.org/

**Note: Asteracea is experimental software.**  
While it appears to work well so far, there likely will be breaking changes to the template syntax.

## Installation

Please use [cargo-edit](https://crates.io/crates/cargo-edit) to always add the latest version of this library:

```cmd
cargo add asteracea
```

## Design goals

* Little boilerplate / Useful defaults

  Most generated boilerplate code is adjusted automatically to what is required. For example, the signature of a component's `.render` method changes if a `Node` is generated.

  [There is still room for improvement here without sacrificing readability.](https://github.com/Tamschi/Asteracea/projects/2)

* Co-location / [DRY]

  Intent shouldn't need to be reiterated in multiple places (split declaration, initialisation and usage).

  For now, short form captures nested in the component templates provide a way to centralise some semantics (similarly to React's Hooks but without their control flow limitations).

  [Further improvements in this area are planned.](https://github.com/Tamschi/Asteracea/projects/1)

  [DRY]: https://en.wikipedia.org/w/index.php?title=Don%27t_repeat_yourself&oldid=972595923

* Robust code

  Element names are statically checked against [`lignin-schema`] by default, but other schemata can be defined similarly. Empty elements like `<br>` cannot contain children.

  Similar checks for attributes and event names are planned.

  [`lignin-schema`]: https://github.com/Tamschi/lignin-schema

* No default runtime

  Asteracea components compile to plain Rust code with (usually) no further dependencies, which helps keep bundles small.

  Use [`lignin-dom`] or [`lignin-html`] to transform rendered `Node` trees into live user interfaces.

  [`lignin-dom`]: https://github.com/Tamschi/lignin-dom
  [`lignin-html`]: https://github.com/Tamschi/lignin-html

## Examples

### Empty component

The most simple (`Node`-rendering) component can be written like this:

```rust
asteracea::component! {
  Empty()()
  [] // Empty node sequence
}

// Render into a bump allocator:
let mut bump = lignin::bumpalo::Bump::new();
assert!(matches!(
  Empty::new().render(&mut bump),
  lignin::Node::Multi(&[]) // Empty node sequence
));
```

### Unit component

If the component body isn't a `Node` expression, the component will return `()` by default and won't require a `Bump` reference to be rendered.

A different return type can be specified after the render argument list.

```rust
asteracea::component! {
  Unit(/* ::new arguments */)(/* .render arguments */) /* -> () */
  {} // Empty Rust block
}

asteracea::component! {
  Offset(base: usize)(offset: usize) -> usize

  |pub base: usize = {base}|; // ²
  { self.base + offset }
}

assert_eq!(Unit::new().render(), ());
assert_eq!(Offset::new(2).render(3), 5);
```

² <https://github.com/Tamschi/Asteracea/issues/2>

### Counter component

For a relatively complex example, see this parametrised counter:

```rust
use asteracea::component;
use std::cell::RefCell;

fn schedule_render() { /* ... */ }

component! {
  pub Counter(initial: i32, step: i32)()

  |value = RefCell::<i32>::new(initial)|; // shorthand capture
  |step: i32 = {step}|; // long form capture, ²

  <div
    "The current value is: " !{*self.value.borrow()} <br>

    <button
      !{self.step} // shorthand bump_format call
      +"click" {
        *self.value.borrow_mut() += self.step;
        schedule_render();
      }
    >
  >
}

impl Counter {
  pub fn value(&self) -> i32 {
    *self.value.borrow()
  }

  pub fn set_value(&self, value: i32) {
    self.value.replace(value);
  }
}
```

² <https://github.com/Tamschi/Asteracea/issues/2>

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## [Code of Conduct](CODE_OF_CONDUCT.md)

## [Changelog](CHANGELOG.md)

## [Planned features](https://github.com/Tamschi/Asteracea/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement+label%3Aaccepted)

## Versioning

Asteracea strictly follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) with the following exceptions:

* The minor version will not reset to 0 on major version changes (except for v1).  
Consider it the global feature level.
* The patch version will not reset to 0 on major or minor version changes (except for v0.1 and v1).  
Consider it the global patch level.

This includes the Rust version requirement specified above.  
Earlier Rust versions may be compatible, but this can change with minor or patch releases.

Which versions are affected by features and patches can be determined from the respective headings in [CHANGELOG.md](CHANGELOG.md).
