# Asteracea

[![Lib.rs](https://img.shields.io/badge/Lib.rs-*-84f)](https://lib.rs/crates/asteracea)
[![Crates.io](https://img.shields.io/crates/v/asteracea)](https://crates.io/crates/asteracea)
[![Docs.rs](https://docs.rs/asteracea/badge.svg)](https://docs.rs/crates/asteracea)

![Rust 1.40.0](https://img.shields.io/static/v1?logo=Rust&label=&message=1.40.0&color=grey)
[![Build Status](https://travis-ci.com/Tamschi/Asteracea.svg?branch=develop)](https://travis-ci.com/Tamschi/Asteracea/branches)
![Crates.io - License](https://img.shields.io/crates/l/asteracea/0.0.1)

[![GitHub](https://img.shields.io/static/v1?logo=GitHub&label=&message=%20&color=grey)](https://github.com/Tamschi/Asteracea)
[![open issues](https://img.shields.io/github/issues-raw/Tamschi/Asteracea)](https://github.com/Tamschi/Asteracea/issues)
[![open pull requests](https://img.shields.io/github/issues-pr-raw/Tamschi/Asteracea)](https://github.com/Tamschi/Asteracea/pulls)
[![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/asteracea.svg)](https://web.crev.dev/rust-reviews/crate/asteracea/)

TODO_README_DESCRIPTION
TODO: Rewrite the following points into a nice README.

* Co-location! You don't have to hunt down logic elsewhere, it's ideally consilely in one place.
* Unspecific errors (i.e. ones that happen on internal tokens and colour the entire macro invocation) are always bugs!
  * Seriously, please report them, ideally with a reproducible sample. I'll fix them to be much more specific if I can reproduce them.

## Installation

Please use [cargo-edit](https://crates.io/crates/cargo-edit) to always add the latest version of this library:

```cmd
cargo add asteracea
```

## Example

```rust
// TODO_EXAMPLE
```

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

## Versioning

Asteracea strictly follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) with the following exceptions:

* The minor version will not reset to 0 on major version changes (except for v1).  
Consider it the global feature level.
* The patch version will not reset to 0 on major or minor version changes (except for v0.1 and v1).  
Consider it the global patch level.

This includes the Rust version requirement specified above.  
Earlier Rust versions may be compatible, but this can change with minor or patch releases.

Which versions are affected by features and patches can be determined from the respective headings in [CHANGELOG.md](CHANGELOG.md).
