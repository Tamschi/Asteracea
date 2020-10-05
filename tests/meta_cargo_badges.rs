#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn travis_ci() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		&format!(
			r#"^travis-ci = \{{ repository = "{user}/{repo}", branch = "{branch}" \}}$"#,
			user = USER,
			repo = REPOSITORY,
			branch = BRANCH,
		)
	);
}

#[test]
fn is_it_maintained_issue_resolution() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		&format!(
			r#"^is-it-maintained-issue-resolution = \{{ repository = "{user}/{repo}" \}}$"#,
			user = USER,
			repo = REPOSITORY,
		)
	);
}

#[test]
fn is_it_maintained_open_issues() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		&format!(
			r#"^is-it-maintained-open-issues = \{{ repository = "{user}/{repo}" \}}$"#,
			user = USER,
			repo = REPOSITORY,
		)
	);
}

#[test]
fn maintenance() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		if BRANCH == "develop" || BRANCH == "unstable" {
			r#"^maintenance = \{ status = "experimental" \}$"#
		} else if BRANCH.starts_with('v') {
			// Stable branch.
			r#"^maintenance = \{ status = "(actively-developed|passively-maintained)" \}$"#
		} else {
			// Just check it's there.
			r#"^maintenance = \{ status = ".+" \}$"#
		}
	);
}
