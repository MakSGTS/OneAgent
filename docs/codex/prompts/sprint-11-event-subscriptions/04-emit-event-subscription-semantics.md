# Task 04: Emit Event Subscription semantics

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

- `docs/Roadmap.md`, Sprint 11 Task 04;
- `docs/architecture/event-subscription-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0012-bsl-symbols-in-semantic-graph.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`.

## Required gate

Proceed only when Tasks 01–03 are committed, all implementation validation has
passed, and the live baseline contains the accepted graph model, parser, and
resolution outcome boundary without task-created uncommitted changes.

## Task

Integrate the committed Event Subscription parser and resolver into the
filesystem semantic graph builder. Emit typed metadata/ownership, source and
handler References, and handler Triggers with deterministic provenance,
diagnostics, statistics, and repeated-build behavior.

## Source contract / production source

Use only the Task 02 parser model and Task 03 resolution outcomes. Resolve after
all supported metadata nodes and BSL declarations exist. Do not reparse XML,
re-extract BSL, infer unsupported source families, or create a second target
resolution authority.

## Scope

One production projection joining committed descriptor facts and resolution
outcomes to committed graph semantics.

## Included

- Add `EventSubscriptions` to supported production discovery through the
  dedicated parser while preserving every other top-level directory path.
- Insert one UUID-backed `Metadata(EventSubscription)` node with common synonym
  and typed event payload.
- Insert exactly one configuration-to-subscription Contains ownership edge.
- Retain parsed observations until all metadata/module/procedure nodes exist,
  then resolve through the committed Task 03 boundary.
- Emit one source References edge per unique resolved subscription/metadata
  target and aggregate every equivalent selector occurrence as deterministic
  provenance.
- Emit one handler References and one handler Triggers edge to the exact owned
  Procedure from one accepted handler outcome.
- Attach ADR-0033 provenance containing descriptor/source role, subscription,
  occurrence or handler, normalized selector/path, target, stable producer,
  origin, confidence, and resolution.
- Project missing, ambiguous, incompatible, invalid-owner, malformed, and
  unsupported outcomes into deterministic typed diagnostics with no placeholder
  facts.
- Record one processed reference-statistics outcome per selector or handler;
  do not count handler References and Triggers separately and do not alter the
  public request ledger.
- Preserve valid nodes/ownership and independent resolved facts after
  recoverable target failures; preserve fatal descriptor behavior with no
  successful partial build result.
- Add generated production-project tests for exact/family/duplicate sources,
  exported/non-exported handlers, failures, reordering, repeated builds, and
  unrelated compatibility.

## Excluded

- New graph/parser/resolver semantics beyond Tasks 01–03.
- Unsupported source metadata kinds, Function handlers, aliases, extensions,
  cross-project/partial workspaces, runtime dispatch, or handler signatures.
- Public ADR-0024 request migration or request-ledger count changes.
- Final provenance-backed fixture, broad consumer/index/Coverage evidence, or
  current-state documentation completion.
- Triggers dependency/Impact reclassification or event-specific Query APIs.

## Acceptance criteria

- A valid project emits exact UUID/name/synonym/event payload, configuration
  ownership, source References, handler References, and handler Triggers.
- Exact selectors resolve one target; bare selectors emit all and only mapped
  kind targets in stable order.
- Manager/object selectors selecting the same object create one edge with all
  sorted provenance rather than duplicate edges.
- Exported and non-exported owned Procedure handlers emit identical relation
  shapes; Function and ownership failures emit no handler edges.
- Malformed/unsupported/missing/ambiguous/incompatible/invalid-owner outcomes
  are typed, counted once per observation, and create no Unknown or placeholder
  fact.
- Valid independent facts survive recoverable resolution failures; fatal
  descriptor errors yield no successful partial build.
- Public reference-request ledger contents/counts remain unchanged.
- Query and Validation observe the accepted facts; Triggers remains excluded
  from dependency/Impact propagation.
- Reordered source and filesystem traversal plus repeated builds produce equal
  graph, payload, edges, provenance, diagnostics, statistics, ledger, report,
  and validation results.
- Existing metadata, BSL, Calls, Reads, Writes, Grants, Includes, Extends,
  Opens, DependsOn, References, diagnostics, and statistics regressions pass.

## Repository Safety

- Recheck Git state, exact Task 01–03 range, builder phases, parser/resolver
  APIs, diagnostics/statistics, tests, Coverage, and applicable `AGENTS.md`.
- Preserve unrelated user files and do not modify the prompt suite.
- Do not add dependencies or commit ignored live source artifacts.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --test event_subscriptions
cargo test -p oneagent-graph --test validation
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Emit Sprint 11 event subscription semantics
```

When authorized, stage only task-owned EDT builder/emission code, focused
production tests, and small generated fixtures required by those tests. Do not
stage planning prompts, ignored live artifacts, final Coverage/docs evidence,
or unrelated paths; do not create an empty commit.

## Final report additions

Report production path, identity/payload/ownership, source and handler relation
inventory, provenance, recoverable/fatal behavior, diagnostics/statistics,
ledger compatibility, determinism, Coverage non-transition, files/tests,
validation, commit hash, final Git status, and the Task 05 gate.
