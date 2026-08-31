# Implement Sprint 39 Change Impact Report

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/diagnostics-engine-implementation.md`
- `docs/codex/templates/diagnostics-engine-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/change-impact-analysis-investigation.md`
- `docs/adr/0017-depends-on-semantics.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0060-git-change-adapter.md`
- `docs/adr/0061-change-impact-analysis.md`

## Prerequisite

Task 2 is committed and ADR-0061 is accepted with no blocking authority,
input, identity, completeness, bound, dependency, or migration question.

## Task

Implement only the ADR-0061 source-independent immutable product Change Impact
report domain over caller-supplied complete previous/current Configuration
graphs and canonical Graph-owned diff/impact results. Do not integrate
Workspace publication or MCP yet.

## Required behavior and evidence

- Place the report domain and evaluator in the exact ADR-selected owner while
  keeping Graph as the only semantic, diff, dependency, propagation, and impact
  authority. Do not reimplement traversal or infer impact from source or
  repository evidence.
- Implement accepted previous/current Configuration identity, report identity,
  affected-item vocabulary, direct/transitive/removed status, availability,
  reasons, observable content, and closed completeness/failure vocabulary.
- Keep identity separate from changeable content. Enforce accepted exact
  duplicate and same-identity/conflicting-content behavior without silently
  selecting an input.
- Produce one deterministic total order independent from caller, Graph
  insertion, hash, filesystem, repository operation, or Configuration discovery
  order. Preserve every accepted Graph reason needed for truthful product
  explanation without creating reverse edges or stored semantic facts.
- Apply accepted input/result/component/depth/reason bounds before unbounded
  cloning or projection. Make rejection, omission, or truncation explicit and
  reconcile checked totals, direct/transitive/removed distributions, returned
  counts, omitted counts, maximum depth, and completeness.
- Implement closed bounded errors that do not echo source paths, repository
  paths/statuses/baselines, Configuration names, node identifiers, provenance,
  source content, credentials, rejected values, or internal error chains except
  where ADR-0061 explicitly accepts a safe typed value.
- Add focused tests for empty/equal, one and mixed direct/transitive/removed,
  added/removed Configuration, each reason/status/availability/completeness,
  exact duplicates, conflicts, reordered inputs, repeated evaluation,
  exact/over bounds, checked-summary overflow paths where constructible,
  inconsistent canonical inputs, and redacted failures.
- Preserve existing public Graph APIs and behavior unless ADR-0061 explicitly
  requires one additive migration. Preserve Workspace, cache, Runtime, MCP,
  Tool Policy, adapters, diagnostics, rules, and Coverage outside this domain.

## Excluded scope

Workspace snapshot storage/publication, cache serialization, watcher/Git input
orchestration, MCP schema or handler changes, product UI, new graph concepts or
propagation rules, source parsing, diagnostics/rules, scoring/risk prediction,
refactoring, source edits, transactions, current-state documentation, and
Sprint completion.

## Validation

Run non-zero focused canonical-input/identity/vocabulary/order/duplicate/
conflict/completeness/summary/bound/error/redaction/reorder/repetition tests and
affected Graph/Analysis package/API/Rustdoc checks, then:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

## Suggested commit message

`Implement Sprint 39 Change Impact report`

## Final report additions

Report domain owner and API, canonical Graph input reuse, report and
Configuration identity, vocabulary, ordering, duplicate/conflict behavior,
completeness, bounds, summaries, failures/redaction, exact focused tests/counts,
dependency/API impact, full-gate results, and deferred Workspace/MCP
integration.
