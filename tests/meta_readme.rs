#![cfg(not(miri))]

#[test]
fn installation() {
	version_sync::assert_contains_regex!("README.md", "^cargo add {name}$");
}

#[test]
fn versioning() {
	version_sync::assert_contains_regex!(
		"README.md",
		r"^`{name}` strictly follows \[Semantic Versioning 2\.0\.0\]"
	);
}
