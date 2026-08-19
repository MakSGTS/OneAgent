# Task 07: Complete Sprint 6 member coverage

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, documentation, comments, Rustdoc, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-emission-task.md`

Read the profile, template, all required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/attribute-tabular-section-source-investigation.md`
- the accepted Sprint 6 ADR created by Task 02;
- `docs/adr/0006-semantic-graph.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0025-references-endpoint-validation.md`

## Required gate

Proceed only when Tasks 03–06 are committed or explicitly proven unnecessary
by the accepted ADR and live evidence. All accepted production behavior must be
implemented before any registry transition. Recheck that no child-created diff
remains and that focused predecessor tests pass.

## Task

Close the production evidence matrix for the accepted Sprint 6 capability,
transition Coverage only where complete evidence exists, recompute live
aggregates, and synchronize current-state documentation. Do not add new semantic
behavior merely to satisfy a registry entry.

## Production evidence scope

- The smallest real-format EDT fixture matrix established by Task 01 and
  expanded only as required by the accepted ADR.
- Full `FileSystemEdtSemanticGraphBuilder` behavior, not isolated parser or
  graph-construction helpers alone.
- Graph/build Validation, Query, Diff, Impact where applicable, request ledger,
  reports, diagnostics, provenance, ordering, source-order independence, and
  repeated builds.

## Included

- Missing positive and negative production-builder evidence for Tasks 03–06.
- UUID-present, UUID-absent, equal-name, nearest-owner, duplicate/conflict,
  invalid-state, malformed, ordering, and repeated-build cases required by the
  ADR.
- Independent graph-domain and EDT Coverage changes only for newly proven
  capabilities or evidence items.
- Representative-test links and limitations grounded in live tests.
- Aggregate counts recomputed from registries and verified by tests.
- Synchronization of `docs/architecture/semantic-model-2.md` and
  `docs/Roadmap.md` with the implemented boundary and remaining limitations.
- Regression evidence for the completed Sprint 3 first slice.

## Excluded

- Architecture changes, parser redesign, new graph semantics, or new reference
  families.
- Coverage status based only on ADR or planning text.
- Marking Sprint 6 completed; Task 08 owns the completion decision.
- Sprint 7 and later scope.
- Unsupported quality percentages, performance claims, or invented counts.

## Acceptance criteria

- Every applicable accepted criterion has a passing focused or production
  integration test and a traceable registry evidence item.
- Registry transitions are independent and change only capabilities whose full
  required evidence is present.
- No `Supported` capability has missing evidence or a contradictory limitation.
- Representative test paths and names exist and execute meaningful tests.
- Aggregate counts are derived from the live registries and their tests pass.
- Production fixtures prove deterministic provenance, ordering, Query,
  Validation, Diff, and repeated builds for the complete accepted slice.
- Existing Attribute/TabularSection, ownership, reference-request, and endpoint
  validation regressions remain green.
- Roadmap and Semantic Model 2 describe implemented and deferred scope without
  marking Sprint 6 complete.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --test coverage
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-edt coverage
cargo test -p oneagent-edt --test ownership
```

Confirm every filter runs meaningful tests. Then run the complete workspace
validation from `docs/codex/core/validation.md`, including `git diff --check`.

## Commit

After successful validation, stage only task-owned fixtures, tests, Coverage,
current-state documentation, and required implementation fixes directly tied to
missing accepted evidence. Create one commit:

```text
Complete Sprint 6 member coverage
```

The current user explicitly authorizes this commit. Never stage the prompt
suite, use `git add .`, or create an empty commit.

## Final report additions

Report the evidence matrix, Coverage transitions, aggregate verification,
remaining limitations, files, tests, validation, commit hash, exact Git status,
and the Task 08 gate.
