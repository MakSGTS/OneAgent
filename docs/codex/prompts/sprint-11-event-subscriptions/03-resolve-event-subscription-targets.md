# Task 03: Resolve Event Subscription targets

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

- `docs/Roadmap.md`, Sprint 11 Task 03;
- `docs/architecture/event-subscription-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0012-bsl-symbols-in-semantic-graph.md`;
- `docs/adr/0016-cross-module-bsl-call-resolution.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0033-event-subscription-semantics.md`.

## Required gate

Proceed only when Tasks 01 and 02 are committed, all required implementation
validation passed, and the live baseline contains the accepted graph model and
typed Event Subscription source model without task-created uncommitted changes.

## Task

Implement deterministic adapter-private resolution outcomes for supported
source selectors and owned Common Module handler procedures. Do not integrate
Event Subscription discovery or emit production nodes/edges yet.

## Source contract / production source

Consume only the committed Task 02 typed source model. Use the committed graph
taxonomy, canonical name/kind indexes, Contains ownership, and declared BSL
Procedure nodes as resolution authorities. Do not reparse XML or BSL.

## Scope

One resolution boundary separating source parsing from later semantic
projection and defining every accepted or rejected outcome.

## Included

- Add a focused `event_subscription_resolution` module or smallest equivalent
  internal boundary.
- Map the exact eleven ADR-0033 serialized prefixes to eight existing metadata
  kinds without broad fallback.
- Resolve qualified selectors by exact canonical metadata name and mapped kind.
- Resolve bare selectors to the complete unique stable-ID-ordered set of graph
  metadata nodes of the mapped kind; zero candidates is missing, not success.
- Preserve distinct manager/object observations even when their target sets
  overlap; aggregate target IDs only in the resolution result intended for
  later edge insertion.
- Resolve `CommonModule.<module>.<procedure>` through exact ownership:
  CommonModule metadata object to Module to Procedure.
- Accept an owned non-exported Procedure; reject Function and never reuse
  exported-only cross-module call resolution as the semantic authority.
- Define typed resolved, missing, ambiguous, incompatible-kind, invalid-owner,
  malformed, and unsupported outcomes with deterministic candidate ordering.
- Define one processed statistics outcome per source selector or handler;
  unsupported/malformed parser outcomes remain a separately named rejected-
  observation path.
- Retain inputs needed for deterministic provenance without creating graph
  edges, diagnostics, or public request-ledger entries in this task.
- Add focused positive, negative, duplicate, reordered, and repeated-resolution
  tests over constructed canonical graph snapshots.

## Excluded

- `EventSubscriptions` directory discovery, metadata node/payload insertion,
  configuration ownership, References/Triggers edge emission, production
  provenance, or build-result diagnostics/statistics integration.
- Public ADR-0024 request category, candidate, identity, lifecycle, report, or
  diff changes.
- Case-insensitive aliases, partial workspaces, extension/cross-project targets,
  unsupported metadata families, Function handlers, or runtime dispatch.
- Tracked production fixture, Coverage transition, or current-state docs.

## Acceptance criteria

- All eleven supported prefixes map only to their eight accepted metadata kinds
  and every unsupported prefix remains unsupported.
- Qualified resolution distinguishes missing, ambiguous, and incompatible kind
  with stable candidates and no placeholder result.
- Bare resolution returns all and only metadata nodes of its mapped kind,
  unique and stable-ID ordered, and treats an empty family as missing.
- Equivalent manager/object selectors may overlap without losing source
  observation context or duplicating target identity.
- Handler resolution proves both Contains relations and exact Procedure kind;
  non-exported is accepted, while Function, missing module/symbol, ambiguity,
  incompatible kind, invalid owner, and malformed path are distinct outcomes.
- Resolution does not mutate the graph, emit edges/diagnostics, increment live
  build statistics, or add public request-ledger values.
- Reordered observations and repeated resolution return equal ordered results.
- Existing SemanticResolutionIndex behavior and tests remain compatible.

## Repository Safety

- Recheck Git state, Task 01–02 commits, resolution APIs, ownership/index
  consumers, diagnostics/statistics types, tests, and applicable `AGENTS.md`.
- Preserve unrelated user files and do not modify the committed prompt suite.
- Do not add dependencies or silently expand public APIs.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --lib event_subscription_resolution::tests
cargo test -p oneagent-graph --test resolution
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Resolve Sprint 11 event subscription targets
```

When authorized, stage only the task-owned resolution module, internal exports,
focused tests, and necessary non-production support types. Do not stage
production builder/emission, fixtures, Coverage/docs completion, planning
prompts, or unrelated paths; do not create an empty commit.

## Final report additions

Report prefix mappings, exact/family policies, handler ownership/export policy,
typed outcome matrix, ordering/aggregation, graph mutation absence, request-
ledger non-migration, files/tests, validation, commit hash, final Git status,
and the Task 04 gate.
