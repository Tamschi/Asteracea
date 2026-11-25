#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn bugs_link() {
	version_sync::assert_contains_regex!(
		"CONTRIBUTING.md",
		&format!(
			r"^\[bugs\]: https://github\.com/{user}/{repo}/issues/new\?assignees=&labels=bug&template=bug_report\.md&title=$",
			user = USER,
			repo = REPOSITORY,
		)
	);
}

#[test]
fn feature_requests_link() {
	version_sync::assert_contains_regex!(
		"CONTRIBUTING.md",
		&format!(
			r"^\[feature requests\]: https://github\.com/{user}/{repo}/issues/new\?assignees=&labels=enhancement&template=feature_request\.md&title=$",
			user = USER,
			repo = REPOSITORY,
		)
	);
}

#[test]
fn custom_issue_link() {
	version_sync::assert_contains_regex!(
		"CONTRIBUTING.md",
		&format!(
			r#"^\["Custom issue"\]: https://github\.com/{user}/{repo}/issues/new\?assignees=&labels=&template=custom_issue\.md&title=$"#,
			user = USER,
			repo = REPOSITORY,
		)
	);
}
