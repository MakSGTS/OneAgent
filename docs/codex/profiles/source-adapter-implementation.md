# Source Adapter Implementation Profile

## Purpose

Use this profile for implementing one bounded multi-artifact source-adapter
ingestion, mapping, or cross-adapter conformance slice.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/context-management.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/parser.md`
- `docs/codex/workflows/source-adapter.md`

## Task-family expectations

- Use repository-owned source evidence or provenance-backed paired fixtures.
- Treat discovery, artifact assembly, parsing, and semantic contribution as
  separate contracts with one explicit orchestration boundary.
- Define complete versus explicitly partial input and typed failure scope.
- Preserve accepted source-independent identities and public graph semantics.
- Prove deterministic end-to-end behavior through production entry points.
- When cross-adapter equivalence is in scope, define canonical comparison
  dimensions and deliberate provenance or source-format differences.
- Do not combine unresolved source-format or semantic architecture with
  implementation; use preceding investigation or architecture tasks.
