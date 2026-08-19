# Task 06: Integrate accepted Sprint 6 member references

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
- `docs/adr/0017-depends-on-semantics.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0025-references-endpoint-validation.md`

## Required gate

Proceed only when Task 05 is committed and the Sprint 6 ADR explicitly accepts
additional member-reference observations or behavior beyond the completed
Attribute/Dimension/Resource metadata-type first slice. If the ADR accepts no
new reference work and the live implementation already satisfies every
applicable criterion, report `already_complete` with exact evidence and do not
create an empty commit.

## Task

Integrate only the additional member-reference source forms accepted by the
Sprint 6 ADR into the public reference-request lifecycle and its deterministic
terminal projections.

## Source contract

Use only Task 04 parser observations backed by Task 01 artifacts and accepted by
Task 02. Existing private EDT projection evidence may retain descriptor paths
and source roles; source-specific types must not leak into `oneagent-graph`.

## Included

- Collection-time `SemanticReferenceRequest` creation with source provenance.
- Stable request identity, sorted expected kinds, candidates, and provenance.
- Exact name-and-kind resolution and explicit complete/partial workspace input.
- Resolved, missing, ambiguous, incompatible-kind, partial, invalid-owner, and
  duplicate outcomes required by the accepted source contract.
- Direct `References` projection and only explicitly justified companion
  `DependsOn` facts.
- Typed diagnostics, statistics derived exactly once, report/build validation,
  and request Diff consistency.
- Precise additive endpoint validation if the accepted contract requires new
  endpoint pairs.
- Positive, negative, duplicate, reordered-source, lifecycle-transition, and
  repeated-build production tests.

## Excluded

- Reference categories or target mappings not accepted by Task 02.
- Placeholder, Unknown, unresolved, ambiguous, or partial target nodes/edges.
- Changing the existing nine-kind metadata-reference behavior without an
  explicit accepted migration.
- BSL calls, query sources, Writes, protected resources, Subsystem content,
  extension targets, forms, commands, and later sprint families.
- Coverage transitions and Sprint status changes.

## Acceptance criteria

- Every accepted observation becomes one canonical request with collection
  provenance before resolution.
- Resolution uses exact accepted kinds and names and preserves explicit
  workspace completeness.
- Terminal requests deterministically drive at most one direct edge or one
  diagnostic projection, with no double-counted statistics.
- Duplicate observations aggregate provenance without duplicating requests,
  edges, diagnostics, or counts.
- Failed and partial outcomes emit no false resolved edge or placeholder node.
- Endpoint validation accepts exactly implemented pairs and rejects all other
  new combinations.
- Request ledger, reports, build validation, and Diff remain consistent.
- Existing metadata-reference and ownership production regressions pass.
- Repeated builds and reordered equivalent observations are identical.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-edt metadata_reference
cargo test -p oneagent-edt --test ownership
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`.

## Commit

After successful validation, stage only task-owned graph/EDT implementation,
fixture, test, and documentation paths and create one commit:

```text
Integrate Sprint 6 member references
```

The current user explicitly authorizes this commit. Never stage the prompt
suite, use broad staging, or create an empty commit.

## Final report additions

Report the source contract, request lifecycle, projection and statistics
behavior, validator impact, remaining gaps, files, tests, validation, commit
hash, exact Git status, and the Task 07 gate.
