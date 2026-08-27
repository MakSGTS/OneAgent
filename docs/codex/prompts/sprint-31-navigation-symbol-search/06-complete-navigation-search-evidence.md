# Complete Sprint 31 Navigation and Search Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/ide-extension-implementation.md`

## Template

`docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0053-navigation-symbol-search.md`
- `docs/architecture/navigation-symbol-search-investigation.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Roadmap.md`
- `.github/workflows/ci.yml`

## Prerequisites / Required gate

Task 5 is committed and every focused source-location, protocol, Runtime,
extension unit, Extension Host, real-process, and package gate passes.

## Task

Complete public, cross-platform, compatibility, dependency, scope, and
current-state evidence for the accepted Sprint 31 boundary.

## Scope

### Included

Clean locked extension build and package; complete source-location/producer/
Graph/Workspace/Tool Policy/MCP/public-process/extension matrix; tracked
positive, negative, missing, ambiguous, incompatible, partial, duplicate,
reordered, repeated-build/invocation, path-escape, Unicode/line-ending,
truncation, cancellation, failure, EOF, and cleanup evidence as applicable;
macOS/Windows CI integration; API/catalog/schema/manifest/package/dependency/
license/generated-artifact/ignored-test/zero-match/secret/path-leak/deferred-
scope audits; and synchronized `docs/Architecture.md`,
`docs/architecture/semantic-model-2.md`, `docs/Roadmap.md`, and extension
user/contributor documentation.

### Excluded

New production behavior, LSP/provider APIs, reference search UI, diagnostics,
chat/context UI, workspace reload/watch, remote/web/multi-root, external-client
compatibility, Marketplace work, telemetry, edits/refactoring, and broad
performance/security claims.

## Acceptance Criteria

- Every accepted ADR-0053 source-location, symbol-query, MCP, Tool Policy, UI,
  path-safety, compatibility, and lifecycle claim has non-zero public evidence.
- Clean CI and local gates cover every accepted Rust and extension boundary
  without weakening prior checks.
- Catalog, schemas, handlers, clients, manifests, tests, packages, and current-
  state documentation agree exactly.
- No unsupported capability or production dependency is claimed.

## Task-specific Validation

- Run the complete clean extension install/typecheck/build/unit/Extension Host/
  real-process/package/inventory gate.
- Run the canonical Rust workspace gate:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo test --workspace`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`.
- Run API, catalog/schema/handler/policy, dependency/license, package,
  generated-artifact, ignored/zero-match, secret/path-leak, deferred-scope,
  Markdown link, and `git diff --check` audits.

## Suggested commit message

`Complete Sprint 31 navigation and search evidence`

## Final report additions

Report exact command/test outcomes, CI platforms, supported symbol/location
matrix, package inventory, dependency and exclusion audits, documentation
transitions, compatibility evidence, and preserved behavior.
