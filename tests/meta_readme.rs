#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn installation() {
	version_sync::assert_contains_regex!("README.md", "^cargo add {name}$");
}

#[test]
fn versioning() {
	version_sync::assert_contains_regex!(
		"README.md",
		&format!(
			r"^{repo} strictly follows \[Semantic Versioning 2\.0\.0\]",
			repo = REPOSITORY,
		)
	);
}
