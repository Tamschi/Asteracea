# Security Policy

## Supported Versions

| Version | Supported          | ***Initial*** Reason for Removal |
| ------- | ------------------ | -------------------------------- |
| 0.0.1   | :white_check_mark: |                                  |

Faulty versions are [yanked](https://doc.rust-lang.org/cargo/commands/cargo-yank.html), where possible after a Semver-compatible update is made available, and added to the table above as unsupported.  
They are also marked with an additional `v….….…-yanked` tag in Git to make them easily recognisable, but original release tags are not removed.

Security advisories are published through [the respective section on this repository here](https://github.com/Tamschi/TODO_CRATE_NAME/security/advisories) and [RustSec/advisory-db](https://github.com/RustSec/advisory-db).

## Reporting a Vulnerability

If you find a security issue, please contact me privately first, so that I can publish a fix before the announcement!

You can reach me via XMPP or email at [tamme@schichler.dev](mailto:tamme@schichler.dev).  
Prefer XMPP and mention "vulnerability" if you'd like an immediate response, though I can't always guarantee this of course.

## Notes

As `0.0.z` versions cannot be upgraded in a Semver-compatible way, these can be yanked without an automatically resolved alternative becoming available.
Should it become impossible to fix a vulnerability in an API-compatible way, an `x.….…` or `0.y.…` version can be yanked entirely, too.

Yanked versions are still available for download, so your builds will not break with an existing `Cargo.lock` file.  
Please include it in your version control (and source release packages for executables). Cargo does this by default.

While there will be advisories about any security issues and undefined behaviour, other bugfix releases are more quiet.  
To be notified automatically, either subscribe to releases through the repository watch feature on GitHub or use for example [Dependabot] with [`package-ecosystem: cargo`](https://docs.github.com/en/code-security/supply-chain-security/keeping-your-dependencies-updated-automatically/configuration-options-for-dependency-updates#package-ecosystem).
To check only for vulnerabilities, use [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit#readme) instead, which is available as a GitHub Action as [actions-rs/audit-check](https://github.com/actions-rs/audit-check#readme).

Once a version becomes yanked/unsupported, please update or upgrade to a supported version in a timely manner.
I'll try to make this as painless as possible where manual changes are required, but a simple [`cargo update -p TODO_CRATE_NAME`](https://doc.rust-lang.org/cargo/commands/cargo-update.html) should do the trick in most cases.

[Dependabot]: https://docs.github.com/en/code-security/supply-chain-security/keeping-your-dependencies-updated-automatically
