#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn homepage() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		&format!(
			r#"^homepage = "https://github\.com/{user}/{repo}/tree/v{{version}}"$"#,
			user = USER,
			repo = REPOSITORY,
		)
	);
}
