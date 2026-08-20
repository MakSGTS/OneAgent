# Task 03: Emit nested Subsystem composition

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

- `docs/Roadmap.md`, Sprint 10 Task 03;
- `docs/architecture/subsystem-hierarchy-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0020-includes-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0032-subsystem-hierarchy-semantics.md`.

## Required gate

Proceed only when Tasks 01 and 02 are committed, all implementation validation
has passed, and the live baseline contains the accepted graph rules and ordered
typed hierarchy source model without task-created uncommitted changes.

## Task

Integrate the committed hierarchy parser into the filesystem semantic graph
builder. Emit nested metadata/flat Subsystem nodes, configuration ownership,
direct hierarchy Includes, and nested direct content Includes with deterministic
provenance and existing diagnostic/statistics behavior.

## Source contract / production source

Use only the Task 02 hierarchy source model and ADR-0032 three-projection
agreement. Nested direct content must flow through the existing
`EdtSubsystemContentReader`, ADR-0020 allowlist/resolution policy, and current
Includes aggregation. Do not reparse or reinterpret XML in the emitter.

## Scope

One production projection joining committed nested source facts to committed
graph semantics.

## Included

- Replace or extend only the Subsystem discovery branch needed to consume the
  recursive ordered source model while preserving every other top-level
  metadata directory path.
- Insert one UUID-backed metadata Subsystem node and one
  `<UUID>:subsystem` flat node per nested descriptor using existing constructors
  and payload/provenance conventions.
- Insert configuration-to-metadata `Contains` for nested metadata objects; do
  not add ownership to flat Subsystem nodes.
- Aggregate and emit one direct parent-flat-to-child-flat Includes edge per
  resolved hierarchy observation.
- Attach ADR-0032 provenance containing both descriptor paths/UUIDs, exact raw
  parent and child fields, resolved flat IDs, stable producer, origin,
  resolution, and confidence.
- Collect nested direct content through the existing ADR-0020 normalization,
  resolution, diagnostics, statistics, provenance, and edge aggregation.
- Preserve recognized-but-deferred `Subsystem.<...>` content behavior.
- Add production tests for depth, duplicate local names, direct hierarchy,
  nested content, source agreement failures, recoverable content failures,
  duplicates, reordered input, and repeated builds.
- Preserve top-level node/edge IDs, direct content facts, diagnostics,
  statistics, unrelated metadata/module behavior, and graph validation.

## Excluded

- New parser grammar, graph kinds, endpoint rules, Query API, or stored
  transitive edges.
- Silent hierarchy repair, partial graph output after fatal hierarchy errors,
  directory-only inference, Subsystem content semantics, aliases, command-
  interface data, or other metadata families.
- Coverage transition, registry aggregate edits, broad consumer/index evidence,
  or final current-state documentation completion.
- Dependency or Impact propagation through Includes.

## Acceptance criteria

- A valid nested fixture emits exact metadata and flat Subsystem IDs, exact
  configuration ownership, and only direct hierarchy/content Includes.
- Equal UUID/source inputs preserve identity across reordering and repeated
  builds; duplicate local names under different parents do not collide.
- Hierarchy edge provenance contains every ADR-0032 context field and is sorted
  and deduplicated independently from input order.
- Nested direct content resolves exactly like top-level ADR-0020 content;
  malformed, unsupported, missing, ambiguous, and incompatible content remains
  recoverable and typed.
- Any hierarchy projection mismatch remains fatal and yields no successful
  partial build result.
- `Subsystem.<...>` content creates no semantic fact and cannot create a
  self-loop.
- No transitive Includes edge is stored; the Task 01 Query derives expected
  membership from direct production facts.
- Existing top-level and unrelated graph facts, diagnostics/statistics, and
  validation regressions pass.

## Repository Safety

- Recheck Git state, exact committed Task 01–02 range, builder integration,
  tests, fixtures, Coverage, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify the prompt suite.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --test subsystem_hierarchy
cargo test -p oneagent-edt --test includes
cargo test -p oneagent-graph --test validation
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Emit Sprint 10 nested subsystem composition
```

When authorized, stage only task-owned EDT integration code, focused production
tests, and fixtures with verified provenance. Do not stage planning prompts,
Coverage/docs completion, or unrelated files; do not create an empty commit.

## Final report additions

Report production path, node/edge identity, ownership, provenance, content
resolution, hierarchy versus content failure behavior, diagnostics/statistics,
fixture evidence, Coverage non-transition, validation, commit hash, final Git
status, and the Task 04 gate.
