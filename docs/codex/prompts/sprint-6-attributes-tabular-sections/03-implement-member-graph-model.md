# Task 03: Implement the Sprint 6 member graph model

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, documentation, comments, Rustdoc, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-model-task.md`

Read the profile, template, all required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/attribute-tabular-section-source-investigation.md`
- the accepted Sprint 6 Attribute and TabularSection ADR created by Task 02;
- `docs/adr/0003-semantic-domain-model.md`
- `docs/adr/0006-semantic-graph.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0023-typed-metadata-payload.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0025-references-endpoint-validation.md`

Locate the Task 02 ADR from its commit instead of assuming a stale ADR number.

## Required gate

Proceed only when Task 02 is committed with message
`Define Sprint 6 member semantics`, the ADR is `Accepted`, and it defines any
required source-independent model changes. If the ADR proves the live model
already satisfies all prerequisites, verify every applicable criterion and
report `already_complete`; do not manufacture code changes or an empty commit.

## Task

Implement only the source-independent metadata and graph-model prerequisites
required by the accepted Sprint 6 ADR. Keep EDT parsing and production graph
emission out of this task.

## Included

- Accepted metadata-domain types or member content required by the ADR.
- Accepted graph representation and controlled constructors or accessors.
- Deterministic identity inputs, equality, ordering, and collision behavior.
- Exact ownership or endpoint schema changes required before producer work.
- Invalid-state rejection without silent repair.
- Query, Diff, Impact, report, and build-validation compatibility required by
  the new source-independent model.
- Focused metadata-domain and graph-domain tests, including regression evidence
  for the completed first slice.
- Public API/Rustdoc updates when the accepted contract requires them.

## Excluded

- EDT XML parsing, fixtures, builder emission, resolution, or Coverage changes.
- New semantics not stated in the accepted ADR.
- Identity changes for existing UUID-backed nodes unless explicitly accepted.
- Placeholder nodes or permissive endpoint fallbacks.
- Forms, commands, and later sprint work.

## Acceptance criteria

- Source-independent ownership remains in metadata/graph crates; no adapter type
  leaks into public domain APIs.
- Identity and equality match the accepted contract and remain deterministic.
- Existing Attribute and TabularSection identities and containment behavior
  regressions pass.
- Validation rejects every newly defined invalid state and preserves exact
  issue ordering.
- Query exposes accepted content or navigation without a parallel adapter API.
- Semantic-content-only changes preserve identity and produce the accepted Diff
  and Impact behavior.
- Public API and serialization impact are documented; absent serialization
  remains explicitly absent.
- No producer or Coverage behavior changes.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test ownership
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`.

## Commit

After all acceptance criteria and validation succeed, stage only task-owned
model, test, and documentation paths and create one commit:

```text
Implement Sprint 6 member graph model
```

The current user explicitly authorizes this commit. Never use `git add .`,
never stage the prompt suite, and do not create an empty commit.

## Final report additions

Report graph-model impact, public API impact, validation/query behavior,
serialization impact, created and modified files, tests, validation, commit
hash, preserved behavior, exact Git status, and the Task 04 gate.
