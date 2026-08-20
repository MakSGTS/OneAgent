# Task 03: Complete Sprint 9 production evidence

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

- `docs/Roadmap.md`, Sprint 9 Task 03;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0019-grants-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0031-conditional-grants-semantics.md`.

## Required gate

Proceed only when Tasks 01 and 02 are committed, all implementation validation
has passed, and the live graph exposes the accepted conditional AccessRight and
Grants production slice without uncommitted task-created changes.

## Task

Complete representative production, generic consumer, index-equivalence,
Coverage-regression, and current-state documentation evidence for ADR-0031.
Do not add another production semantic capability.

## Source contract / production source

Use the committed Task 02 full EDT Grants builder path and the real conditional
and unconditional `Rights.rights` artifacts. Generated temporary cases may
exercise negative matrices, but repository-owned artifacts remain the positive
provenance oracle.

## Scope

One evidence-completion boundary proving the integrated feature through all
affected consumers while synchronizing current-state documentation.

## Included

- Add any missing representative assertions for typed payload lookup, generic
  Query navigation, node/edge Diff, reverse Impact, reports, validation,
  complete Semantic Index, and incremental clean-rebuild equivalence.
- Prove conditional/unconditional distinction, deterministic ordering,
  repeated builds, provenance aggregation, and unchanged unrelated semantic
  facts through the production builder.
- Recheck existing Metadata, Calls, Reads, Writes, Includes, Extends, Opens,
  DependsOn, Grants, typed payload, Query, Diff, Impact, report, and index tests.
- Synchronize `docs/architecture/semantic-model-2.md` from planned architecture
  to implemented current state.
- Update graph and EDT Coverage evidence or limitations only where required to
  describe the expanded supported slice; retain every status and exact aggregate
  count unless live registry calculation proves otherwise.
- Keep Sprint 9 `next` or `active`; do not mark it completed before review.

## Excluded

- New graph or parser behavior, resource families, diagnostic/request types,
  condition evaluator/query service, deny, inheritance, defaults, profiles,
  groups, users, effective authorization, or persistence/transport surfaces.
- New Coverage capabilities, manual aggregate edits unsupported by registry
  output, or Sprint 10 planning.
- Fixes unrelated to conditional direct Grants evidence.

## Acceptance criteria

- Every applicable ADR-0031 completion criterion has executable evidence.
- Generic consumers observe conditional node IDs and payload without a
  condition-specific semantic authority or ordering difference.
- Complete and incremental index results match clean rebuilds for add, remove,
  and replace cases affecting conditional AccessRight nodes and their edges.
- Full-builder evidence covers present and absent restrictions, exact condition
  content, provenance, diagnostics/statistics, deterministic duplicate and
  repeated behavior, and unrelated semantic compatibility.
- Graph and EDT Coverage registries retain their committed statuses and exact
  aggregate counts; tests state the expanded evidence boundary.
- Semantic Model and Roadmap current-state text match live committed behavior,
  while deny, evaluation, inheritance, defaults, profiles, groups, users, and
  effective authorization remain deferred.
- Full workspace validation succeeds.

## Repository Safety

- Recheck Git state, exact committed Task 01–02 range, tests, consumers,
  Coverage, docs, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify the prompt suite.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test grants
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Complete Sprint 9 production evidence
```

When authorized, stage only task-owned evidence tests and synchronized
current-state documentation. Do not stage planning prompts or unrelated files;
do not create an empty commit.

## Final report additions

Report the evidence matrix, consumer/index equivalence, unchanged and expanded
production behavior, Coverage statuses/counts, documentation synchronization,
remaining deferred scope, validation, commit hash, final Git status, and the
Task 04 gate.
