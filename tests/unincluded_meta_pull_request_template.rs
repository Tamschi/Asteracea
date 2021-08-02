#![cfg(not(miri))]

#[test]
fn crate_name() {
	version_sync::assert_contains_regex!(
		".github/PULL_REQUEST_TEMPLATE.md",
		"^Thank you for your contribution to the `{name}` repository!$"
	);
}
