# Complete Sprint 14 Conformance Evidence

Continue OneAgent development.

## Reporting

- Repository content and commit message: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/source-adapter-implementation.md`

## Template

`docs/codex/templates/source-adapter-task.md`

## Authoritative documents

- `docs/adr/0036-designer-xml-adapter.md`
- `docs/architecture/designer-xml-source-investigation.md`
- `docs/architecture/designer-xml-source-corpus.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Roadmap.md`, Sprint 14 execution plan

## Prerequisites / Required gate

Require committed Task 6 production builder, successful preceding full
validation, and clean task-owned state.

## Task

Complete the executable paired EDT/Designer conformance, negative, determinism,
consumer, index, provenance, Coverage, and current-state documentation evidence
for the exact ADR-0036 first slice without changing its semantics.

## Source evidence / paired fixtures

Add the smallest tracked paired fixture allowed by corpus policy. Record exact
live paths, raw SHA-256 values, reduction/export treatment, reduced hashes, and
why the pair represents the same configuration facts. Do not require ignored
corpora in CI.

## Scope

### Included

- A non-empty canonical projection over accepted identities, kinds, names,
  payloads, ownership, BSL declarations, terminal outcomes, and public consumer
  results.
- Explicit exclusion of ADR-0036 deliberate differences only: source paths,
  producer identifiers, serialization order, encoding/line endings, and other
  exact accepted dimensions.
- One controlled semantic change proving the oracle detects inequality.
- Missing/malformed/unsupported/conflict/partial, source-reordering,
  repeated-build, Query, Diff, report, Validation, complete-index, and
  incremental clean-rebuild evidence applicable to the slice.
- Truthful adapter-specific Coverage evidence and synchronization of Semantic
  Model, Roadmap current state, and fixture documentation.

### Excluded

- New parser/emitter semantics, expanded graph model, whole-graph equality,
  deferred source families, Sprint completion, release review, and prompt
  retirement.

## Acceptance Criteria

- Both fixtures produce non-empty accepted projections and the canonical
  projections are equal before the controlled change.
- The controlled change produces the exact expected difference; no empty or
  filtered-away oracle can pass.
- Every deliberate difference is documented and no semantic identity/content
  dimension is silently excluded.
- Complete and incremental indexes match clean rebuilds for accepted transitions.
- Coverage status/counts derive from executable evidence and documentation
  agrees with code and tests.
- Full validation passes with no ignored-corpus dependency or unrelated change.

## Repository Safety

Modify only task-owned fixtures/tests/Coverage/current-state docs. Preserve
`.codex/`, existing suites, ignored corpora, and implementation semantics.

## Task-specific Validation

- Focused Designer adapter, filesystem/workspace, BSL, graph, conformance,
  Coverage, complete-index, and incremental-index tests with non-zero matches.
- Recompute every fixture hash.
- Run the complete workspace validation gate.

## Suggested commit message

`Complete Sprint 14 conformance evidence`

## Final report additions

Report fixture provenance, canonical projection, deliberate differences,
controlled change, conformance matrix, coverage state, documentation, exact
validation, commit, and Git state.
