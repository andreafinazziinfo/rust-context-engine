# Security Policy

## Supported Versions

Only the latest published release on [crates.io](https://crates.io/crates/rtk-context-engine) receives security fixes. There is no long-term support branch.

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Instead, use [GitHub Security Advisories](https://github.com/andreafinazziinfo/rust-context-engine/security/advisories/new) to report privately. Include:

- Affected crate(s) and version
- Reproduction steps or a minimal example
- Impact (e.g. local privilege escalation, data exposure, DoS)

You should get an initial response within 5 business days. Confirmed vulnerabilities will be fixed and disclosed via a GitHub Security Advisory and a `RUSTSEC` advisory where applicable.

## Automated Scanning

Every CI run executes:

- `cargo audit` (via `actions-rust-lang/audit`) against `Cargo.lock`
- CodeQL static analysis (Rust + GitHub Actions)

Dependabot opens PRs automatically for vulnerable or outdated dependencies.

## Secrets & Publishing

Publishing to crates.io and creating GitHub releases both run through GitHub Actions using repository secrets (`CARGO_REGISTRY_TOKEN`, `GITHUB_TOKEN`). No credentials are stored in the repository. The crates.io token is scoped to this project's crates and should be rotated if a maintainer's GitHub account access is revoked or a token leak is suspected.
