#![cfg(not(miri))]

#[path = "meta_constants_.rs"]
mod constants;
use constants::*;

#[test]
fn user_appears() {
	version_sync::assert_contains_regex!("CODEOWNERS", &format!("^* @{user}$", user = USER));
}
