# Task 05: Complete Sprint 11 production evidence

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

- `docs/Roadmap.md`, Sprint 11 Task 05;
- `docs/architecture/event-subscription-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`.

## Required gate

Proceed only when Tasks 01–04 are committed, all required implementation
validation passed, and the live production builder emits the accepted Event
Subscription slice without task-created uncommitted changes.

## Task

Complete provenance-backed source, generic consumer, complete/incremental
index, Coverage, aggregate-count, and current-state documentation evidence for
ADR-0033. Do not add another source or semantic capability.

## Source contract / production source

Create one tracked reduced fixture under the existing EDT test-fixture
conventions. Its README must trace every selected field to exact live,
root-ignored `OneAgent_EDTproject/src/EventSubscriptions/` and referenced
metadata/Common Module artifacts. Record source paths, selected fragments,
source hashes, and reduced-artifact hashes so the fixture remains auditable if
the live corpus changes or is absent. Generated cases may cover negative and
transition matrices but must not replace this positive production evidence.

## Scope

One evidence-completion boundary proving the integrated feature through every
affected consumer while synchronizing implemented current-state documentation.

## Included

- Include at least one exact supported selector, one bare family selector, two
  manager/object selectors selecting the same metadata object, one unsupported
  selector, one exported handler, one non-exported handler, and representative
  event/synonym values from the live corpus.
- Prove exact metadata/payload/ownership, source References, handler References,
  handler Triggers, deterministic provenance, diagnostics, statistics, and
  unchanged public request-ledger behavior.
- Prove direct generic Query for nodes/ownership/References/Triggers and no
  event-specific second query authority.
- Prove payload, source, handler, and relation add/remove/modify Diff behavior;
  report distributions; validation; dependency/Impact policy; repeated and
  reordered builder equality.
- Prove complete and incremental Semantic Index clean-rebuild equivalence for
  subscription add/remove, event payload change, source add/remove/retarget,
  and handler retarget.
- Recheck existing Metadata, Contains, Calls, References, Reads, Writes,
  Grants, Includes, Extends, Opens, DependsOn, Query, Diff, Impact, reports,
  validation, request ledger, and index behavior.
- Add or update graph-domain and EDT Coverage capabilities/evidence only where
  the live registry requires it for the new MetadataKind and EdgeKind.
- Recompute all graph/EDT aggregate counts from executable registry state;
  never copy planning estimates.
- Synchronize `docs/architecture/semantic-model-2.md` and Sprint 11 Roadmap
  current-state text while keeping Sprint 11 `next` or `active`, not completed.

## Excluded

- New parser, resolver, endpoint, payload, or edge semantics.
- Unsupported metadata families, public multi-target request lifecycle,
  Function handlers, Triggers dependency/Impact propagation, runtime dispatch,
  or event-specific Query APIs.
- Unproven Coverage rows, manual aggregate edits, Sprint 12 planning, previous-
  suite retirement, or unrelated fixes.

## Acceptance criteria

- Every applicable ADR-0033 graph and EDT Coverage completion criterion has
  executable evidence.
- The tracked fixture README proves exact live derivation and hashes for every
  reduced metadata, Event Subscription, Common Module, and BSL artifact.
- The fixture proves exact/family/overlap/unsupported selectors plus exported
  and non-exported handler behavior without hidden dependency on ignored files.
- Query, Diff, report, Validation, dependency/Impact policy, reference ledger,
  repeated builds, and reordered inputs match ADR-0033.
- Complete and incremental indexes match clean rebuilds for every required
  add/remove/modify/retarget transition.
- Coverage statuses, required evidence, limitations, and aggregate counts are
  derived from passing live registry tests and agree across graph, EDT,
  Semantic Model, Roadmap, and review inputs.
- Unsupported source families and every ADR-0033 exclusion remain deferred.
- Full workspace validation succeeds.

## Repository Safety

- Recheck Git state, exact Task 01–04 range, live source provenance, consumers,
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
cargo test -p oneagent-edt --test event_subscriptions
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Complete Sprint 11 production evidence
```

When authorized, stage only task-owned evidence tests/fixtures, Coverage
registry/evidence updates, and synchronized current-state documentation. Do not
stage prompt suites, ignored live files, Sprint 12 work, or unrelated paths; do
not create an empty commit.

## Final report additions

Report acceptance evidence matrix, fixture source provenance/hashes, consumer
and index transitions, ledger/dependency compatibility, Coverage statuses and
derived counts, documentation synchronization, deferred scope, files/tests,
validation, commit hash, final Git status, and the Task 06 gate.
