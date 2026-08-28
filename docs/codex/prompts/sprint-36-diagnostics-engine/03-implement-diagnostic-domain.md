# Implement Sprint 36 Diagnostic Domain

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/diagnostics-engine-implementation.md`
- `docs/codex/templates/diagnostics-engine-task.md`

## Required workflow

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0058-diagnostics-engine.md`

## Prerequisite

Task 2 is committed and ADR-0058 is accepted with no blocking domain,
dependency, or migration question.

## Task

Implement only the ADR-0058 source-independent diagnostic domain: stable
identity, normalized result, suppression outcome, summary, report, bounds, and
deterministic ordering. Do not orchestrate production inputs yet.

## Required behavior and evidence

- Place every public type and invariant in the exact ADR-selected owner and
  preserve dependency direction. Do not duplicate Graph semantic authority.
- Implement the accepted closed vocabularies, validated constructors,
  identifiers, origin/location representation, result status, suppression
  evidence, summaries, reports, errors, accessors, equality, and total ordering.
- Enforce every accepted string, collection, count, identifier, location, and
  report bound at construction. Rejected input must fail deterministically and
  must not leak rejected content through errors or `Debug`.
- Preserve exact distinctions between active, suppressed, omitted, invalid, and
  unavailable evidence required by ADR-0058; do not infer a source location or
  provenance that the input does not own.
- Add focused tests for empty and valid values, every vocabulary member,
  invalid/missing/duplicate/collision cases, exact and one-over bounds,
  suppression outcomes, summary arithmetic, deterministic reordering,
  equality, error redaction, and repeated construction.
- Update only public exports and Rustdoc required by this domain. Keep Graph
  producers, validation execution, Workspace composition, cache, Runtime,
  protocols, fixtures, and current-state documentation outside this task.

## Excluded scope

Input orchestration, adapters, new diagnostics or graph facts, Workspace/cache,
MCP/LSP projection, Rules Engine registry or dynamic execution, diagnostics UI,
Cargo dependency additions, current-state docs, and Sprint completion.

## Validation

Run non-zero focused tests for every new domain type and bound, affected package
tests and API/Rustdoc checks, then:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

## Suggested commit message

`Implement Sprint 36 diagnostic domain`

## Final report additions

Report the domain owner and public API, identity/order/suppression/bound/report
behavior, exact focused tests and counts, dependency impact, full-gate results,
and deferred orchestration.
