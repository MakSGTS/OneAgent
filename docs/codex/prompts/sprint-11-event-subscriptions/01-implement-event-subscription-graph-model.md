# Task 01: Implement Event Subscription graph model

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-model-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 11 Task 01;
- `docs/architecture/event-subscription-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`.

## Required gate

Proceed only when ADR-0033, the source investigation, Sprint 11 Roadmap plan,
Semantic Model planning text, and complete prompt suite are one committed
immutable planning baseline and Sprint 11 is the unique live target.

## Task

Implement only the source-independent metadata, payload, and graph rules
accepted by ADR-0033. Keep EDT parsing, target resolution, discovery, production
emission, fixtures, diagnostics/statistics, and final Coverage transitions
outside this task.

## Scope

One coherent public graph-model boundary making Event Subscription facts typed,
validatable, queryable, and deterministic before a producer can emit them.

## Included

- Add `MetadataKind::EventSubscription` and stable `event_subscription` code
  without changing existing variant codes or identities.
- Add closed typed Event Subscription payload with one non-empty event name;
  enforce exact payload-kind compatibility through metadata and graph APIs.
- Preserve compatibility constructors and public node identity; payload is
  semantic content only.
- Add `EdgeKind::Triggers` with only
  `Metadata(EventSubscription) --Triggers--> Procedure` accepted.
- Extend References validation only for EventSubscription-to-supported-source-
  metadata and EventSubscription-to-Procedure pairs from ADR-0033.
- Update every exhaustive MetadataKind, MetadataSpecificPayload, and EdgeKind
  consumer with explicit behavior rather than wildcarding away new semantics.
- Prove equality, payload-only Diff, generic Query, Validation, reports,
  complete index, incremental index, Coverage-model enumeration, and stable
  ordering.
- Prove Triggers remains outside dependency and Impact classification while
  References retains its existing dependency behavior.
- Update public Rustdoc only where the new public model changes contracts.

## Excluded

- EDT XML parsing, directory discovery, source/handler resolution, graph
  insertion, provenance producers, diagnostics, statistics, or fixtures.
- Public ADR-0024 request categories or lifecycle changes.
- New source metadata families, Function handlers, runtime execution, event-
  specific Query methods, serialization, or persistence.
- Final EDT/graph Coverage status and aggregate-count transitions.

## Acceptance criteria

- Existing MetadataKind and EdgeKind codes, equality, ordering, Query, Diff,
  Impact, Validation, report, and index behavior remain compatible.
- Event Subscription payload accepts only EventSubscription metadata nodes;
  wrong-kind payloads are rejected deterministically.
- A payload-only event change preserves node ID and reports one modified
  semantic-content node rather than removal/addition.
- All eight source-metadata families plus Procedure validate as References
  targets; all reversed, unsupported, Unknown, Function, Module, AccessRight,
  and unrelated pairs fail.
- The one Triggers pair validates; every other pair fails deterministically.
- Triggers edges have stable standard identity, are visible in generic Query,
  Diff, report, and indexes, and never become dependency/Impact edges.
- Empty, reversed insertion, repeated construction, complete-index, and
  incremental-transition cases are deterministic.
- No EDT production behavior or final Coverage status changes in this task.

## Repository Safety

- Recheck Git state, public enum consumers, metadata/graph tests, Coverage, and
  applicable `AGENTS.md` before editing.
- Preserve unrelated user files, especially
  `docs/codex/prompts/run-next-sprint.md` and
  `docs/roadmap-calendar-forecast.md` when still outside scope.
- Do not add dependencies or modify the committed prompt suite.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Implement Sprint 11 event subscription graph model
```

When authorized, stage only task-owned metadata/graph code, affected exhaustive
consumer updates, focused tests, and necessary Rustdoc. Do not stage EDT
production code, planning prompts, or unrelated paths; do not create an empty
commit.

## Final report additions

Report metadata/payload model, public API impact, enum migration, endpoint
matrices, identity/equality/diff behavior, Query/Validation/index behavior,
dependency/Impact exclusion, Coverage non-transition, files/tests, validation,
commit hash, final Git status, and the Task 02 gate.
