# Task 04: Complete Sprint 12 production evidence

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-emission-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 12 Task 04;
- `docs/architecture/report-data-composition-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`;
- `docs/adr/0034-report-data-composition-semantics.md`.

## Required gate

Proceed only when Tasks 01–03 are committed, all required implementation
validation passed, and the live production builder emits the accepted Report
Data Composition slice without task-created uncommitted changes.

## Task

Complete provenance-backed source, generic consumer, complete/incremental index,
Coverage, aggregate-count, and current-state documentation evidence for
ADR-0034. Do not add another source or semantic capability.

## Source contract / production source

Create one tracked reduced fixture under the existing EDT test-fixture
conventions. Its README must trace every selected Report descriptor and `.dcs`
artifact to exact live root-ignored `OneAgent_EDTproject/src/Reports/` paths.
Record source hashes, selected/reduced treatment, and reduced-artifact hashes so
the fixture remains auditable if the live corpus changes or is absent. Generated
cases may cover negatives and transitions but must not replace positive
production evidence.

## Scope

One evidence-completion boundary proving the integrated feature through every
affected consumer while synchronizing implemented current-state documentation.

## Included

- Include Query, Object, Union, empty main, non-main, nested-deferred, and
  folder-deferred shapes from the source investigation's representative matrix.
- Prove exact Schema/DataSet/Field/Query identity, typed payload, immediate
  ownership, provenance, diagnostics, statistics, and unchanged public request
  ledger/query-relation behavior.
- Prove generic Query navigation for every new node/owner and no DCS-specific
  second query authority.
- Prove main/data-set kind/source/field path/query text add/remove/modify Diff
  behavior; report distributions; validation; Impact exclusion; reordered and
  repeated builder equality.
- Prove complete and incremental Semantic Index clean-rebuild equivalence for
  Schema/DataSet/Field/Query add/remove, main-role change, payload/content
  change, ownership change, and deferred-observation transitions.
- Recheck existing Metadata, Contains, Calls, References, Reads, Writes,
  Grants, Includes, Extends, Opens, Triggers, DependsOn, Query, Diff, Impact,
  reports, validation, request ledger, and index behavior.
- Add or update graph-domain and EDT Coverage capabilities/evidence only where
  live registries require them for the accepted nodes and ownership producers.
- Recompute all graph/EDT aggregate counts from executable registry state;
  never copy planning estimates.
- Synchronize `docs/architecture/semantic-model-2.md` and Sprint 12 Roadmap
  current-state text while keeping Sprint 12 `next` or `active`, not completed.

## Excluded

- New parser, identity, payload, ownership, diagnostic, statistics, query
  grammar, request, or relation semantics.
- Nested Union entities, field folders, partial Query sources, virtual tables,
  batches, temporary tables, field lineage, runtime composition, or non-Report
  schema sources.
- Unproven Coverage rows, manual aggregate edits, Sprint 13 planning,
  previous-suite retirement, or unrelated fixes.

## Acceptance criteria

- Every applicable ADR-0034 graph and EDT Coverage completion criterion has
  executable evidence.
- The fixture README proves exact live derivation and hashes for every reduced
  Report descriptor and DCS artifact.
- The fixture proves Query/Object/Union/empty/non-main positive behavior and
  nested/folder deferred behavior without hidden dependency on ignored files.
- Query, Diff, report, Validation, Impact policy, request ledger, absence of
  query-source relations, repeated builds, and reordered inputs match ADR-0034.
- Complete and incremental indexes match clean rebuilds for every required
  add/remove/modify/ownership/deferred transition.
- Coverage statuses, required evidence, limitations, and aggregate counts are
  derived from passing live registry tests and agree across graph, EDT,
  Semantic Model, Roadmap, and review inputs.
- Unsupported source constructs and every ADR-0034 exclusion remain deferred.
- Full workspace validation succeeds.

## Repository Safety

- Recheck Git state, exact Task 01–03 range, live source provenance, consumers,
  indexes, Coverage, docs, tests, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify or retire prompt suites.
- Do not stage ignored live project artifacts; add only the reduced tracked
  fixture with explicit derivation evidence.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test report_data_composition
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Complete Sprint 12 production evidence
```

When authorized, stage only task-owned evidence tests/fixtures, Coverage
registry/evidence updates, and synchronized current-state documentation. Do not
stage prompt suites, ignored live files, Sprint 13 work, or unrelated paths; do
not create an empty commit.

## Final report additions

Report acceptance evidence matrix, fixture source provenance/hashes, consumer
and index transitions, request-ledger/query-relation compatibility, Coverage
statuses and derived counts, documentation synchronization, deferred scope,
files/tests, validation, commit hash, final Git status, and the Task 05 gate.
