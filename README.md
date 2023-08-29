# Asteracea

[![Lib.rs](https://img.shields.io/badge/Lib.rs-*-84f)](https://lib.rs/crates/asteracea)
[![Crates.io](https://img.shields.io/crates/v/asteracea)](https://crates.io/crates/asteracea)
[![Docs.rs](https://docs.rs/asteracea/badge.svg)](https://docs.rs/asteracea)

![Rust 1.58](https://img.shields.io/static/v1?logo=Rust&label=&message=1.58&color=grey)
[![CI](https://github.com/Tamschi/Asteracea/workflows/CI/badge.svg?branch=develop)](https://github.com/Tamschi/Asteracea/actions?query=workflow%3ACI+branch%3Adevelop)
![Crates.io - License](https://img.shields.io/crates/l/asteracea/0.0.2)

[![GitHub](https://img.shields.io/static/v1?logo=GitHub&label=&message=%20&color=grey)](https://github.com/Tamschi/Asteracea)
[![open issues](https://img.shields.io/github/issues-raw/Tamschi/Asteracea)](https://github.com/Tamschi/Asteracea/issues)
[![open pull requests](https://img.shields.io/github/issues-pr-raw/Tamschi/Asteracea)](https://github.com/Tamschi/Asteracea/pulls)
[![good first issues](https://img.shields.io/github/issues-raw/Tamschi/Asteracea/good%20first%20issue?label=good+first+issues)](https://github.com/Tamschi/Asteracea/contribute)

[![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/asteracea.svg)](https://web.crev.dev/rust-reviews/crate/asteracea/)
[![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252FAsteracea)](https://iteration-square.schichler.dev/#narrow/stream/project.2FAsteracea)

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

- Little boilerplate / Useful defaults

  Most generated boilerplate code is adjusted automatically to what is required. For example, the signature of a component's `.render` method changes if a `Node` is generated.

  [There is still room for improvement here without sacrificing readability.](https://github.com/Tamschi/Asteracea/projects/2)

- Co-location / [DRY]

  Intent shouldn't need to be reiterated in multiple places (split declaration, initialisation and usage).

  For now, short form captures nested in the component templates provide a way to centralise some semantics (similarly to React's Hooks but without their control flow limitations).

  [Further improvements in this area are planned.](https://github.com/Tamschi/Asteracea/projects/1)

  [DRY]: https://en.wikipedia.org/w/index.php?title=Don%27t_repeat_yourself&oldid=972595923

- Robust code

  Element names are statically checked against [`lignin-schema`] by default, but other schemata can be defined similarly. Empty elements like `<br>` cannot contain children.

  Similar checks for attributes and event names are planned.

  [`lignin-schema`]: https://github.com/Tamschi/lignin-schema

- No default runtime

  Asteracea components compile to plain Rust code with few dependencies, which helps keep bundles small.

  Use [`lignin-dom`] or [`lignin-html`] to transform rendered `Node` trees into live user interfaces.

  [`lignin-dom`]: https://github.com/Tamschi/lignin-dom
  [`lignin-html`]: https://github.com/Tamschi/lignin-html

## Examples

Additional examples can be found [in the examples directory](examples#list-of-examples).

### Empty component

The most simple (`Node`-rendering) component can be written like this:

```rust
asteracea::component! {
  pub Empty()() -> Sync
  [] // Empty node sequence
}

// Render into a bump allocator:
// This is generally only this explicit at the application root.
let mut bump = bumpalo::Bump::new();
let root = {
  struct Root;
  rhizome::sync::Node::new(core::any::TypeId::of::<Root>())
};
assert!(matches!(
  Box::pin(Empty::new(root.as_ref(), Empty::new_args_builder().build()).unwrap())
    .as_ref()
    .render(&mut bump, Empty::render_args_builder().build())
    .unwrap(),
  lignin::Node::Multi(&[]) // Empty node sequence
));
```

VDOM [`Sync`-ness](https://doc.rust-lang.org/stable/std/marker/trait.Sync.html) can be inferred (even transitively) at zero runtime cost by omitting `-> Sync` (or `-> !Sync`), except for components visible outside their crate.

### Unit component

A return type other than `Node` can be specified after the render argument list:

```rust
asteracea::component! {
  Unit(/* ::new arguments */)(/* .render arguments */) -> ()
  {} // Empty Rust block
}

asteracea::component! {
  Offset(base: usize)(offset: usize) -> usize

  let pub self.base: usize = base; // ²
  { self.base + offset }
}

// This is generally only this explicit at the application root.
let mut bump = bumpalo::Bump::new();
let root = {
  struct Root;
  rhizome::sync::Node::new(core::any::TypeId::of::<Root>())
};
assert_eq!(
  Box::pin(Unit::new(root.as_ref(), Unit::new_args_builder().build()).unwrap())
    .as_ref()
    .render(&mut bump, Unit::render_args_builder().build())
    .unwrap(),
  (),
);
assert_eq!(
  Box::pin(Offset::new(root.as_ref(), Offset::new_args_builder().base(2).build()).unwrap())
    .as_ref()
    .render(&mut bump, Offset::render_args_builder().offset(3).build())
    .unwrap(),
  5,
);
```

² <https://github.com/Tamschi/Asteracea/issues/2>

### Counter component

For a relatively complex example, see this parametrised counter:

```rust
use asteracea::component;
use lignin::web::Event;
use std::cell::Cell;

fn schedule_render() { /* ... */ }

component! {
  pub Counter(
    /// The counter's starting value.
    initial: i32,
    priv step: i32, // field from argument
    pub enabled: bool = true, // default parameter
  )(
    // optional argument;
    // `class` is `Option<&'bump str>` only inside this component, not its API.
    class?: &'bump str,
  ) -> !Sync // visible across crate-boundaries, so use explicit `Sync`ness

  // shorthand capture; Defines a struct field.
  let self.value = Cell::<i32>::new(initial);

  <div
    // conditional attribute from `Option<&'bump str>`
    ."class"? = {class}

    // Anything within curlies is plain Rust.
    "The current value is: " !(self.value()) <br>

    <button
      ."disabled"? = {!self.enabled} // boolean attribute from `bool`
      "+" !(self.step) // shorthand `bump_format` call
      on bubble click = Self::on_click_plus
    >
  >
}

// Counter is a plain struct, so `impl` works as expected!
impl Counter {
  pub fn value(&self) -> i32 {
    self.value.get()
  }

  pub fn set_value(&self, value: i32) {
    self.value.set(value);
  }

  // This may alternatively take a `*const Self` or `Pin<&Self>`.
  // Inline handlers are also possible, but not much less verbose.
  fn on_click_plus(&self, _: Event) {
    self.value.set(self.value() + self.step);
    schedule_render();
  }
}


asteracea::component! {
  CounterUser()()

  <*Counter
    *initial = {0} // parameters by name
    *step = {1}
    .class = {"custom-counter"} // without Some(…)
  >
}
```

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING](CONTRIBUTING.md) for more information.

## [Code of Conduct](CODE_OF_CONDUCT.md)

## [Changelog](CHANGELOG.md)

## [Planned features](https://github.com/Tamschi/Asteracea/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement+label%3Aaccepted)

## Versioning

Asteracea strictly follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) with the following exceptions:

- The minor version will not reset to 0 on major version changes (except for v1).  
Consider it the global feature level.
- The patch version will not reset to 0 on major or minor version changes (except for v0.1 and v1).  
Consider it the global patch level.

This includes the Rust version requirement specified above.  
Earlier Rust versions may be compatible, but this can change with minor or patch releases.

Which versions are affected by features and patches can be determined from the respective headings in [CHANGELOG.md](CHANGELOG.md).

Note that dependencies of this crate may have a more lenient MSRV policy!
Please use `cargo +nightly update -Z minimal-versions` in your automation if you don't generate Cargo.lock manually (or as necessary) and require support for a compiler older than current stable.
