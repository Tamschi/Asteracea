#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn bug_report() {
	version_sync::assert_contains_regex!(
		".github/ISSUE_TEMPLATE/bug_report.md",
		&format!(r"^- `rustc --version`: \[e\.g\. {}\]$", RUST_VERSION)
	);

	version_sync::assert_contains_regex!(
		".github/ISSUE_TEMPLATE/bug_report.md",
		r"^- Crate version \(if applicable\): \[e\.g\. {version}\]$"
	);
}
