# Integrate Sprint 31 VS Code Navigation and Search

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
- official pinned VS Code 1.134.0 APIs selected by ADR-0053

## Prerequisites / Required gate

Task 4 is committed; its public Runtime/MCP matrix and full Rust gate pass; the
extension package and lifecycle baselines remain compatible.

## Task

Integrate the accepted semantic symbol-search and source-navigation experience
into the public VS Code extension.

## Scope

### Included

Accepted manifest commands and demand activation; editor-independent validation
of navigation/search results; Runtime client request API with cancellation and
bounded validation; explicit connection-state gating; Quick Pick symbol input,
ordering, labels/details, ambiguity, empty/no-result/truncated behavior; safe
workspace-relative URI resolution and containment; document opening,
zero-based selection/reveal conversion; stable bounded user-visible failures;
context/disposable ownership; repetition and deactivation cleanup; pure unit,
pinned Extension Host, real-process, and package-inventory evidence.

### Excluded

TypeScript-owned semantic matching/ranking, parsing opaque provenance,
filesystem search, source content, implicit or automatic Runtime connection,
definition/reference/document/workspace-symbol providers, LSP, diagnostics,
chat/context UI, edits/refactoring, remote/web/multi-root, Marketplace work,
telemetry, Runtime installation, and unrelated lifecycle changes.

## Acceptance Criteria

- Commands activate only on explicit demand and use the accepted public MCP
  contract; no semantic behavior is recreated in TypeScript.
- Search, selection, cancellation, no-result, ambiguity, truncation, invalid or
  escaping location, missing file, process/protocol failure, repetition, and
  cleanup follow ADR-0053 deterministically.
- Workspace-relative paths cannot escape the selected root; line/column
  coordinates convert exactly and Unicode/line-ending cases are covered.
- Every command, Quick Pick, request, cancellation source, listener, and editor
  resource has one bounded owner and is disposed on deactivation.
- Non-zero Extension Host and real-process tests exercise the public command to
  opened-document/selection boundary.

## Task-specific Validation

- Run non-zero extension typecheck, build, unit, pinned Extension Host, and
  real-process tests.
- Run the complete package/VSIX inventory gate and required focused/full Rust
  checks for the consumed public protocol.
- Audit manifest/source/test agreement, activation, disposables, path safety,
  redaction, deferred API absence, and `git diff --check`.

## Suggested commit message

`Integrate Sprint 31 VS Code navigation and search`

## Final report additions

Report commands and activation, Quick Pick behavior, MCP ownership, coordinate
and path safety, failure/cancellation UI, Extension Host and real-process test
counts, lifecycle cleanup, package impact, and preserved Sprint 30 behavior.
