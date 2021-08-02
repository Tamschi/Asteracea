#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn changelog() {
	// This will become less useful with patches, so I'm on the lookout for a crate that lets me test major, minor and revision independently.
	version_sync::assert_contains_regex!("CHANGELOG.md", "^## {version}$");
}

#[test]
fn html_root_url() {
	version_sync::assert_contains_regex!(
		"src/lib.rs",
		r#"^#!\[doc\(html_root_url = "https://docs\.rs/{name}/{version}"\)\]$"#
	);
}

#[test]
fn homepage() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		&format!(
			r#"^homepage = "https://github\.com/{0}/{{name}}/tree/v{{version}}"$"#,
			USER,
		)
	);
}

#[test]
fn documentation() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		r#"^documentation = "https://docs\.rs/{name}/{version}"$"#
	);
}
