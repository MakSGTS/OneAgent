# Complete Sprint 30 VS Code Extension Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/ide-extension-implementation.md`

## Template

`docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/architecture/vscode-extension-foundation-investigation.md`
- `docs/Architecture.md`
- `docs/Roadmap.md`
- `.github/workflows/ci.yml`

## Prerequisites / Required gate

Task 5 is committed and all focused package, client, real-process, and
extension-host gates pass.

## Task

Complete public, packaging, CI, dependency, compatibility, and current-state
evidence for the accepted Sprint 30 boundary.

## Scope

### Included

Cross-platform pinned Node/package-manager CI integrated without weakening Rust
CI; clean lockfile install; typecheck/build/unit/extension-host/real-process
matrix; deterministic VSIX creation and full inventory assertion; public
positive/negative/bounds/failure/repeated/cleanup matrix; manifest/source/test
agreement; dependency/license/generated-artifact audits; ignored/zero-match and
secret/path leakage audits; and synchronized `docs/Architecture.md`,
`docs/Roadmap.md`, and extension user/contributor documentation.

### Excluded

New production behavior, navigation, LSP, diagnostics, chat, EDT, publication,
signing, telemetry, remote/web hosts, performance/security claims, and unrelated
Rust or Coverage changes.

## Acceptance Criteria

- A clean checkout can install, typecheck, build, test, and package with pinned
  inputs on every accepted CI platform.
- Every manifest contribution and connection state has executable evidence;
  all required test commands run non-zero cases.
- The VSIX inventory is exact and excludes every prohibited file class.
- Documentation truthfully distinguishes implemented and deferred scope.
- No new production dependency exists unless separately approved and recorded.

## Task-specific Validation

- Run the complete extension clean-install/typecheck/build/unit/extension-host/
  real-process/package/inventory gate.
- Run the canonical Rust workspace gate:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo test --workspace`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`.
- Run dependency, ignored-test, zero-match, secret/path, generated-artifact,
  manifest/source, package-inventory, link, and `git diff --check` audits.

## Suggested commit message

`Complete Sprint 30 VS Code extension evidence`

## Final report additions

Report CI platforms, exact command/test outcomes, package inventory, dependency
and exclusion audits, documentation transitions, and preserved behavior.
