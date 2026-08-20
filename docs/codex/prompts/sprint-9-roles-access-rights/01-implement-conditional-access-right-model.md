# Task 01: Implement the conditional AccessRight graph model

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

- `docs/Roadmap.md`, Sprint 9 Task 01;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0019-grants-semantics.md`;
- `docs/adr/0031-conditional-grants-semantics.md`.

## Required gate

Proceed only when ADR-0031, the Sprint 9 Roadmap plan, Semantic Model
synchronization, and complete prompt suite are one committed immutable planning
baseline and Sprint 9 is the unique live target.

## Task

Implement only the source-independent conditional AccessRight model accepted by
ADR-0031. Keep EDT parser, resolver, graph emission, production fixtures,
Coverage transitions, and documentation completion outside this task.

## Scope

One additive graph-domain payload and identity contract with complete generic
consumer compatibility evidence.

## Included

- Add typed AccessRight payload content compatible only with
  `NodeKind::AccessRight` and expose it through a typed `GraphNode` accessor.
- Add an opaque row-restriction value that trims outer Unicode whitespace,
  rejects empty canonical text, and preserves all remaining content exactly.
- Preserve `AccessRight::new` and every unconditional ID and display name.
- Add an optional-restriction construction path; append the exact
  length-delimited canonical condition only for conditional identity.
- Store typed payload through `SemanticGraph::insert_access_right`.
- Prove equality, ordering, collision separation, validation, Query, Diff,
  Impact, reports, complete Semantic Index, incremental index, and graph
  Coverage declarations remain deterministic.
- Update graph Rustdoc only where the public model changes.

## Excluded

- New NodeKind, EdgeKind, endpoint pair, ownership relation, serialization or
  persisted-data format, authorization query API, or condition evaluator.
- EDT reader/builder code, source parsing, resolution, diagnostics, statistics,
  provenance production, fixtures, or edge emission.
- Deny, defaults, inheritance, profiles, groups, users, effective access, or
  condition-language equivalence.
- Coverage capability/status/count changes.

## Acceptance criteria

- Existing unconditional AccessRight IDs and names are byte-for-byte unchanged.
- Equal resource/right/condition inputs create equal typed payload and identity
  regardless of construction order or provenance.
- Conditional versus unconditional and two distinct conditions never collide.
- Leading/trailing whitespace normalizes deterministically; whitespace-only
  input returns a typed error; internal content remains exact.
- AccessRight payload is accepted only for `NodeKind::AccessRight`; all unrelated
  kinds reject it without changing existing Metadata and member payload rules.
- Inserted AccessRight nodes expose typed content, while legacy payload-free
  AccessRight nodes remain compatible.
- Node diffs detect payload changes; Query, Impact, reports, complete and
  incremental indexes preserve identities, payload, ordering, and results.
- No EDT behavior or Coverage aggregate changes.

## Repository Safety

- Recheck Git state, exact definitions, public consumers, tests, and applicable
  `AGENTS.md` before editing.
- Preserve unrelated user files, especially
  `docs/codex/prompts/run-next-sprint.md` and
  `docs/roadmap-calendar-forecast.md` when still outside scope.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --lib access_right::tests
cargo test -p oneagent-graph --lib node::tests
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --test coverage
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Implement Sprint 9 conditional access rights
```

When authorized, stage only task-owned graph code, graph tests, and necessary
Rustdoc. Do not stage planning prompts or unrelated files; do not create an
empty commit.

## Final report additions

Report graph model and public API impact, exact identity compatibility, payload
validation, consumer behavior, serialization impact, files/tests, validation,
commit hash, final Git status, and the Task 02 gate.
