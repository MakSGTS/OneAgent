# Implement Sprint 32 LSP Diagnostics

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/implementation.md`

## Template

`docs/codex/templates/implementation-task.md`

## Authoritative documents

- `docs/adr/0054-lsp-adapter.md`
- `docs/architecture/lsp-adapter-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0053-navigation-symbol-search.md`

## Prerequisites / Required gate

Task 5 is committed and its semantic/public-process matrix passes. The accepted
LSP document identity, URI, position, and immutable handler APIs are available.

## Task

Implement the bounded recoverable diagnostic projection accepted by ADR-0054.

## Scope

### Included

Accepted document diagnostic request/notification behavior and capability;
canonical code, severity, message, source, location, ordering, identity and
result identifiers; diagnostic-to-document projection only from typed or
accepted source-node evidence; deterministic omission of unlocatable or
ambiguous diagnostics; exact bounds and truncation/unchanged behavior if
accepted; positive/negative EDT and empty Designer evidence; malformed,
missing, conflicting, repeated and reordered cases; public process behavior;
and preservation of Graph diagnostic identity and MCP diagnostics.

### Excluded

New diagnostic rules or orchestration, guessed source positions, source reads,
opaque provenance parsing, workspace diagnostics unless accepted explicitly,
push/pull mode not selected by ADR-0054, mutable document sync, diagnostics UI,
suppression/configuration, edits/code actions, MCP changes, and new dependencies
unless approved.

## Acceptance Criteria

- LSP diagnostics are a deterministic bounded projection of existing canonical
  recoverable diagnostics and never become a second diagnostic authority.
- Codes, severities, messages, URIs/ranges, ordering, identity, empty/unchanged
  behavior, and unsupported-location handling exactly follow ADR-0054.
- Existing Graph reports/cache, MCP diagnostic results, and adapter builds
  remain compatible.
- Non-zero focused and public-process tests cover every accepted diagnostic
  behavior, lifecycle, repetition, and deferred-capability absence.

## Task-specific Validation

- Run focused Graph diagnostic/report/cache, EDT/Designer, Workspace, MCP
  diagnostic regression, protocol LSP, Runtime handler, and public process tests.
- Audit diagnostic capability/handler/code/severity/location/bound agreement.
- Run the canonical Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 32 LSP diagnostics`

## Final report additions

Report diagnostic projection ownership, supported source coverage, code/
severity/location/ordering/bounds, omitted cases, public test counts, and
preserved Graph/MCP behavior.
