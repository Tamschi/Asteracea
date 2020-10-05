#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn rust_version() {
	version_sync::assert_contains_regex!(".travis.yml", &format!(r"^  - {}$", RUST_VERSION));
}
