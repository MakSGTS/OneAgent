# Task 01: Implement Subsystem hierarchy graph rules

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

- `docs/Roadmap.md`, Sprint 10 Task 01;
- `docs/architecture/subsystem-hierarchy-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0020-includes-semantics.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0032-subsystem-hierarchy-semantics.md`.

## Required gate

Proceed only when ADR-0032, the source investigation, Sprint 10 Roadmap plan,
Semantic Model planning text, and complete prompt suite are one committed
immutable planning baseline and Sprint 10 is the unique live target.

## Task

Implement only the source-independent graph rules accepted by ADR-0032: the
additive direct hierarchy endpoint, deterministic cycle validation, and the
read-only transitive metadata-membership Query projection. Keep EDT parsing,
discovery, emission, fixtures, production provenance, and Coverage transitions
outside this task.

## Scope

One coherent graph-model boundary making accepted hierarchy facts valid and
queryable before any producer can emit them.

## Included

- Allow `NodeKind::Subsystem --Includes--> NodeKind::Subsystem` while preserving
  every ADR-0020 metadata endpoint and rejecting all other Includes pairs.
- Reject hierarchy self-loops and deterministic directed cycles consisting only
  of Subsystem-to-Subsystem Includes edges.
- Add one Query method with the exact ADR-0032 transitive metadata-membership
  behavior and a name consistent with the live public Query API.
- Return unique metadata members ordered by stable node identity; use Subsystem
  nodes only as traversal intermediates; return empty for missing or wrong-kind
  inputs; remain cycle-safe over an invalid graph.
- Keep traversal source-independent and derived from canonical direct edges;
  create no stored closure, provenance, diagnostics, or secondary authority.
- Prove direct edge identity, Validation, Query, Diff, reports, complete Semantic
  Index, incremental index equivalence, and repeated query determinism.
- Prove Includes remains excluded from dependency and Impact traversal.
- Update graph Rustdoc only where public endpoint or Query behavior changes.

## Excluded

- New NodeKind or EdgeKind variants, hierarchy payloads, persisted closure, or
  serialization/persistence formats.
- EDT source parsing, recursive discovery, graph emission, provenance producer,
  fixtures, diagnostics/statistics, or content resolution.
- Dedicated hierarchy index state independent from canonical graph/index data.
- Dependency, usage, or Impact reclassification.
- Coverage capability/status/count changes and current-state completion docs.

## Acceptance criteria

- Existing metadata-member Includes edges validate exactly as before.
- Flat Subsystem hierarchy is the only newly accepted Includes endpoint; every
  unrelated endpoint remains invalid.
- Self-loop and multi-node hierarchy cycles produce stable validation issues;
  valid shared metadata membership is not misclassified as a cycle.
- Transitive membership includes direct and descendant metadata members exactly
  once, excludes every Subsystem intermediary and metadata Subsystem target,
  and is stable across insertion order.
- Missing/wrong-kind input returns empty; invalid cycles cannot loop forever.
- Canonical graph edge count contains only direct facts; Query creates no
  transitive Includes edge.
- Complete and incremental index-backed Query behavior equals clean graph Query
  behavior after hierarchy/member add, remove, and replacement changes.
- Dependency queries and Impact results remain unchanged.
- No EDT behavior or Coverage aggregate changes.

## Repository Safety

- Recheck Git state, exact graph definitions, public consumers, tests, and
  applicable `AGENTS.md` before editing.
- Preserve unrelated user files, especially
  `docs/codex/prompts/run-next-sprint.md` and
  `docs/roadmap-calendar-forecast.md` when still outside scope.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --lib incremental_index::tests
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Implement Sprint 10 subsystem hierarchy graph rules
```

When authorized, stage only task-owned graph code, graph tests, and necessary
Rustdoc. Do not stage planning prompts, EDT files, or unrelated paths; do not
create an empty commit.

## Final report additions

Report endpoint and cycle rules, public Query API impact, transitive result
contract, stored-versus-derived behavior, dependency/Impact compatibility,
files/tests, validation, commit hash, final Git status, and the Task 02 gate.
