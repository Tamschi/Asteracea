#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn readme() {
	version_sync::assert_contains_regex!("README.md",
		&format!(
			"^# {repo}$",
			repo = REPOSITORY,
		)
	);
}

#[test]
fn changelog() {
	version_sync::assert_contains_regex!("CHANGELOG.md",
		&format!(
			"^# {repo} Changelog$",
			repo = REPOSITORY,
		)
	);
}
