# Task 05: Emit Sprint 6 member ownership

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, documentation, comments, Rustdoc, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-emission-task.md`

Read the profile, template, all required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/attribute-tabular-section-source-investigation.md`
- the accepted Sprint 6 ADR created by Task 02;
- `docs/adr/0006-semantic-graph.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0023-typed-metadata-payload.md`

## Required gate

Proceed only when Task 04 is committed with message
`Parse Sprint 6 EDT member semantics`, its parser output implements the accepted
contract, and the graph-model prerequisites are present. If parser evidence is
partial or incompatible with the ADR, make no emission changes.

## Task

Map the accepted Task 04 parser observations to canonical semantic member nodes
and exactly one immediate-owner `Contains` fact per accepted member. Preserve
the existing node-collection-before-ownership-emission ordering boundary.

## Production source

- the accepted `EdtMetadataChildDescriptor` or its Task 04 successor;
- the EDT graph builder and child contribution path in `adapters/edt/src/lib.rs`;
- repository-owned real-format fixtures identified by Task 01.

## Included

- Canonical node kind and identity mapping for accepted member observations.
- Exactly one immediate-owner containment edge with deterministic identity.
- Collection ordering that does not depend on nested XML completion order.
- Node and edge provenance using accepted source context.
- Missing-owner, invalid-owner, duplicate-owner, collision, and conflicting
  observation behavior.
- Query owner/child navigation, graph/build validation, Diff, and optional
  Impact ownership traversal required by the accepted contract.
- Positive, negative, duplicate-name, UUID/UUID-less, reordered-source, and
  repeated-build production tests.

## Excluded

- New reference categories, target mappings, request resolution, or statistics.
- A second metadata-object owner for an attribute owned by a TabularSection.
- Placeholder or Unknown nodes and silent invalid-state repair.
- Copying subordinate member facts into top-level metadata payload.
- Coverage transitions and Sprint status changes.
- Forms, commands, and later sprint scope.

## Acceptance criteria

- Every accepted member uses the Task 02 identity contract without adapter-only
  identity drift.
- Top-level and nested ownership uses the accepted nearest owner and passes the
  precise `Contains` schema.
- Nested attributes have no companion metadata-object containment edge.
- Equal names under distinct owners remain independently queryable.
- Node and edge provenance is non-empty, deterministic, and does not
  participate in identity.
- Invalid and duplicate ownership is rejected or diagnosed according to the
  accepted contract, never silently repaired.
- Query, Validation, Diff, and applicable Impact behavior is deterministic.
- Reordered equivalent source and repeated production builds are equal.
- Existing ownership fixtures and completed first-slice behavior remain green.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-edt --test ownership
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-graph --test impact
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`.

## Commit

After successful validation, stage only task-owned producer, fixture, test, and
documentation paths and create one commit:

```text
Emit Sprint 6 member ownership
```

The current user explicitly authorizes this commit. Never stage the prompt
suite, use `git add .`, or create an empty commit.

## Final report additions

Report the production path, identity strategy, provenance strategy, ownership
behavior, remaining gaps, files, tests, validation, commit hash, exact Git
status, and the Task 06 gate.
