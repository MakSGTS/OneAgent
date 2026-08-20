# Task 02: Emit conditional direct role grants

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

- `docs/Roadmap.md`, Sprint 9 Task 02;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0019-grants-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0031-conditional-grants-semantics.md`.

## Required gate

Proceed only when Task 01 is committed and its typed payload, conditional
identity, unconditional compatibility, and full validation are present in the
live baseline.

## Task

Propagate the already parsed optional EDT row restriction through the existing
private role-grant resolution and insertion pipeline, then emit deterministic
conditional AccessRight nodes, companion References, and Grants.

## Source contract / production source

- `adapters/edt/tests/fixtures/role_rights/BaseUser/Rights.rights` contains
  `restrictionByCondition/condition` with `WHERE NOT DeletionMark`.
- `adapters/edt/tests/fixtures/grants_project/src/Roles/BaseUser/Rights.rights`
  provides the corresponding full-builder source.
- `EdtRoleRightDeclaration::row_restriction()` is the typed parser authority.
- Only explicit `true` declarations for currently accepted protected-resource
  kinds are eligible; the condition is opaque and does not affect resolution.

## Scope

One production projection slice joining the existing typed parser observation
to the committed Task 01 graph model.

## Included

- Carry optional canonical condition through `ResolvedRoleGrantObservation`.
- Include it in access-right, References, and Grants aggregation keys and target
  lookup.
- Construct conditional or unconditional AccessRight nodes through the accepted
  graph API.
- Record restriction absence/presence and canonical condition in deterministic
  node/edge provenance source IDs.
- Add focused production tests for real conditions, absent conditions,
  conditional/unconditional separation, identical and distinct conditions,
  duplicates, reordered declarations, false values, missing/ambiguous/
  incompatible/unsupported targets, parser errors, and repeated builds.
- Preserve existing diagnostics, reference statistics, endpoint validation, and
  unrelated graph facts.

## Excluded

- RLS expression parsing, validation, evaluation, or semantic equivalence.
- Deny inference, default/inheritance policy, profiles, groups, users, role
  aggregation, and effective authorization.
- Public reference-request migration or new request/diagnostic categories.
- New protected-resource families, placeholder nodes, direct Role-to-Metadata
  grants, or endpoint changes.
- Coverage transitions, aggregate changes, and final documentation completion.

## Acceptance criteria

- The real fixture exposes typed `WHERE NOT DeletionMark` payload on both proven
  Catalog.Product conditional rights.
- Existing unconditional resource/right AccessRight IDs remain exact.
- Conditional and unconditional declarations for the same role/resource/right
  create distinct nodes and Grants; different conditions remain distinct.
- Identical observations aggregate deterministic provenance and do not duplicate
  semantic nodes or edges; reordered and repeated builds have empty diffs.
- Every conditional AccessRight retains exactly one companion References target
  with accepted provenance.
- False declarations emit no authorization fact; missing, ambiguous,
  incompatible, unsupported, and malformed outcomes remain typed and
  deterministic.
- Reference statistics count accepted resolution attempts exactly as before;
  condition content does not create another attempt.
- Existing graph validation and all unconditional Grants regressions pass.

## Repository Safety

- Recheck Git state, source model, builder pipeline, fixtures, tests, Coverage,
  consumers, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify the prompt suite.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt role_rights
cargo test -p oneagent-edt --test grants
cargo test -p oneagent-graph --test validation
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Emit Sprint 9 conditional role grants
```

When authorized, stage only task-owned EDT production code, focused tests, and
fixtures proven necessary by the task. Do not stage planning prompts or
unrelated files; do not create an empty commit.

## Final report additions

Report the production path, identity and aggregation keys, provenance strategy,
diagnostics/statistics, fixture evidence, Coverage non-transition, validation,
commit hash, final Git status, and the Task 03 gate.
