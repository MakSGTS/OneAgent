# Task 04: Complete Sprint 10 production evidence

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

- `docs/Roadmap.md`, Sprint 10 Task 04;
- `docs/architecture/subsystem-hierarchy-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0020-includes-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0032-subsystem-hierarchy-semantics.md`.

## Required gate

Proceed only when Tasks 01–03 are committed, all implementation validation has
passed, and the live builder emits the accepted nested hierarchy/content slice
without task-created uncommitted changes.

## Task

Complete representative source, generic consumer, complete/incremental index,
Coverage-regression, and current-state documentation evidence for ADR-0032.
Do not add another source or semantic capability.

## Source contract / production source

Create a tracked reduced fixture with an exact derivation README tracing every
selected field to the live, root-ignored
`OneAgent_EDTproject/src/Subsystems/` source artifacts. Record source paths,
selected fragments, and hashes so the fixture remains auditable after the live
reference tree changes or is absent. Generated cases may exercise negative and
transition matrices, but must not replace this provenance-backed positive
fixture.

## Scope

One evidence-completion boundary proving the integrated feature through all
affected consumers while synchronizing implemented current-state documentation.

## Included

- Prove repository-observed depths 1–5, direct parent-child relations, duplicate
  local names, nested direct content, and observed deferred Subsystem self-
  content through bounded executable evidence.
- Add missing assertions for direct generic edge queries and the Task 01
  transitive membership Query, including shared members and stable ordering.
- Prove no transitive edge is persisted and no Includes relation enters
  dependency or Impact traversal.
- Prove node/edge/provenance Diff, report distributions, validation, and
  repeated/reordered builder equality.
- Add complete and incremental Semantic Index clean-rebuild equivalence for
  hierarchy/member add, remove, reparent, and content replacement.
- Recheck existing Metadata, Contains, Includes, Calls, References, Reads,
  Writes, Grants, Extends, Opens, DependsOn, Query, Diff, Impact, reports,
  validation, and index behavior.
- Synchronize `docs/architecture/semantic-model-2.md` from accepted planning
  language to implemented current state.
- Update graph/EDT Coverage evidence or limitations only where live registry
  contracts require it; derive statuses and aggregate counts from executable
  registry output rather than expectation.
- Keep Sprint 10 `next` or `active`; do not mark it completed before review.

## Excluded

- New parser grammar, graph endpoints, query semantics, hierarchy-aware
  dependency/Impact, command-interface data, Subsystem content semantics,
  unsupported content prefixes, or unrelated metadata families.
- New Coverage capability rows unless the live registry model proves the
  existing capabilities cannot truthfully represent the implemented slice.
- Manual aggregate edits unsupported by registry output, Sprint 11 planning,
  or previous prompt-suite retirement.
- Fixes unrelated to nested Subsystem evidence.

## Acceptance criteria

- Every applicable ADR-0032 Coverage completion criterion has executable
  evidence.
- The tracked provenance-backed fixture proves exact vocabulary, depth,
  duplicate-name, and provenance inputs; every reduced/generated case traces
  its derivation clearly.
- Generic Query returns exact direct and transitive results without another
  semantic authority or ordering difference.
- Complete and incremental index results match clean rebuilds for add, remove,
  reparent, and content changes.
- Full-builder evidence covers deterministic source agreement, hierarchy and
  content provenance, recoverable versus fatal outcomes, repeated builds, and
  unrelated semantic compatibility.
- Graph and EDT Coverage registries remain truthful; any status/count change is
  derived and justified by the live capability model.
- Semantic Model and Roadmap text match committed behavior while every
  ADR-0032 exclusion remains deferred.
- Full workspace validation succeeds.

## Repository Safety

- Recheck Git state, exact committed Task 01–03 range, source provenance,
  consumers, Coverage, docs, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify or retire prompt suites.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test subsystem_hierarchy
cargo test -p oneagent-edt --test includes
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Complete Sprint 10 production evidence
```

When authorized, stage only task-owned evidence tests/fixtures, Coverage
evidence, and synchronized current-state documentation. Do not stage prompt
suites or unrelated files; do not create an empty commit.

## Final report additions

Report the evidence matrix, source provenance, direct/transitive consumer and
index equivalence, stored fact inventory, Coverage statuses/counts,
documentation synchronization, remaining deferred scope, validation, commit
hash, final Git status, and the Task 05 gate.
