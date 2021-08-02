#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn weak_assert_branch() {
	let info = git_info::get();

	if let Some(branch) = info.current_branch {
		if branch.contains("HEAD detached") {
			eprintln!("Branch assert ignored: HEAD detached")
		} else if branch == "(no branch)" {
			// Most likely a release tag.
			eprintln!(r#"Branch assert ignored: "(no branch)""#)
		} else if branch == "template" {
			eprintln!("Branch assert ignored: Currently on template branch")
		} else if branch.contains('-') || branch.contains('/') {
			eprintln!("Branch assert ignored: Probably a feature branch")
		} else {
			assert_eq!(BRANCH, branch);
		}
	} else {
		eprintln!("Branch assert ignored: No branch information available")
	}
}

#[test]
fn lib() {
	version_sync::assert_contains_regex!(
		"README.md",
		r"^\[!\[Lib\.rs\]\(https://img\.shields\.io/badge/Lib\.rs-\*-84f\)\]\(https://lib\.rs/crates/{name}\)$"
	);
}

#[test]
fn crates() {
	version_sync::assert_contains_regex!(
		"README.md",
		r"^\[!\[Crates\.io\]\(https://img\.shields\.io/crates/v/{name}\)\]\(https://crates\.io/crates/{name}\)$"
	);
}

#[test]
fn docs() {
	version_sync::assert_contains_regex!(
		"README.md",
		r"^\[!\[Docs\.rs\]\(https://docs\.rs/{name}/badge\.svg\)\]\(https://docs\.rs/{name}\)$"
	);
}

#[test]
fn rust_version() {
	version_sync::assert_contains_regex!(
		"README.md",
		&format!(
			r"^!\[Rust {0}\]\(https://img\.shields\.io/static/v1\?logo=Rust&label=&message={0}&color=grey\)$",
			RUST_VERSION,
		)
	);
}

#[test]
fn build_status() {
	version_sync::assert_contains_regex!(
		"README.md",
		&format!(
			r"^\[!\[CI\]\(https://github\.com/{user}/{{name}}/workflows/CI/badge\.svg\?branch={branch}\)\]\(https://github\.com/{user}/{{name}}/actions\?query=workflow%3ACI\+branch%3A{branch}\)$",
			user = USER,
			branch = BRANCH,
		)
	);
}

#[test]
fn license() {
	version_sync::assert_contains_regex!(
		"README.md",
		r"^!\[Crates\.io - License\]\(https://img\.shields\.io/crates/l/{name}/{version}\)$"
	);
}

#[test]
fn git_hub() {
	version_sync::assert_contains_regex!(
		"Cargo.toml",
		&format!(r#"^repository = "https://github.com/{0}/{{name}}"$"#, USER,)
	);

	version_sync::assert_contains_regex!(
		"README.md",
		&format!(
			r"^\[!\[GitHub\]\(https://img\.shields\.io/static/v1\?logo=GitHub&label=&message=%20&color=grey\)\]\(https://github\.com/{0}/{{name}}\)$",
			USER,
		)
	);
}

#[test]
fn issues() {
	version_sync::assert_contains_regex!(
		"README.md",
		&format!(
			r"^\[!\[open issues\]\(https://img\.shields\.io/github/issues-raw/{0}/{{name}}\)\]\(https://github\.com/{0}/{{name}}/issues\)$",
			USER,
		)
	);
}

#[test]
fn pulls() {
	version_sync::assert_contains_regex!(
		"README.md",
		&format!(
			r"^\[!\[open pull requests\]\(https://img\.shields\.io/github/issues-pr-raw/{0}/{{name}}\)\]\(https://github\.com/{0}/{{name}}/pulls\)$",
			USER,
		)
	);
}

#[test]
fn crev() {
	version_sync::assert_contains_regex!(
		"README.md",
		r"^\[!\[crev reviews\]\(https://web.crev.dev/rust-reviews/badge/crev_count/{name}.svg\)\]\(https://web\.crev\.dev/rust-reviews/crate/{name}/\)$"
	);
}
