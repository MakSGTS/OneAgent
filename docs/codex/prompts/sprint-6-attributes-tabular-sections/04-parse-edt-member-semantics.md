# Task 04: Parse accepted EDT member semantics

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, documentation, comments, Rustdoc, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/parser-implementation.md`

## Template

`docs/codex/templates/parser-task.md`

Read the profile, template, all required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/attribute-tabular-section-source-investigation.md`
- the accepted Sprint 6 ADR created by Task 02;
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0024-reference-request-provenance.md`

## Required gate

Proceed only when the accepted Sprint 6 ADR defines an executable EDT source
contract and all required Task 03 model prerequisites are committed or proven
unnecessary. Recheck the repository-owned source artifacts named by Task 01.
If they are absent or contradict the ADR, make no parser changes and report the
blocker.

## Task

Implement only the accepted EDT Attribute and TabularSection parser contract.
Keep graph insertion, reference resolution, and Coverage transitions out of
this task.

## Source evidence

Start with the real artifacts and generated test inputs recorded by Task 01,
the existing `adapters/edt/src/metadata_structure.rs` reader, and the ownership
fixture under `adapters/edt/tests/fixtures/ownership_project/`. Do not infer
additional XML elements or fields from names in the Roadmap.

## Included

- Parsing only the source forms accepted by the Sprint 6 ADR.
- Source UUID preservation and the accepted owner-scoped UUID-less fallback.
- Immediate-owner observations and nearest-owner behavior.
- Accepted member content and direct/composite type observations.
- Typed malformed, missing, optional, unsupported, duplicate, conflicting, and
  invalid-nesting outcomes required by the ADR.
- Deterministic output independent from equivalent source observation order.
- Realistic positive, negative, duplicate, ordering, and repeated-read tests.
- Parser API/Rustdoc changes required by the accepted contract.

## Excluded

- Graph node or edge insertion, reference resolution, diagnostics projection,
  statistics, Coverage, or Roadmap status changes.
- Speculative deeper nesting, tabular-section references, new target mappings,
  or unproven member fields.
- Forms, commands, Designer XML, and later sprint work.
- Changes to accepted source-independent identity or ownership semantics.

## Acceptance criteria

- Every parsed field maps to evidence cited by Task 01 and semantics accepted by
  Task 02.
- UUID-present and UUID-absent identities are deterministic and owner-scoped as
  required.
- Equal names under different accepted owners do not collide.
- Duplicate and conflicting observations follow one explicit typed policy.
- Missing names/identifiers, malformed values, unsupported nesting, and unknown
  source forms are retained as errors or typed outcomes; none become false
  semantic facts.
- Equivalent reordered input and repeated reads produce equal parser output.
- Existing Attribute, TabularSection, Dimension, Resource, Form, and Command
  parser regressions remain green.
- The parser emits no graph facts.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-edt metadata_structure
cargo test -p oneagent-edt --test ownership
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`.

## Commit

After successful validation, stage only task-owned parser, fixture, test, and
documentation paths and create one commit:

```text
Parse Sprint 6 EDT member semantics
```

The current user explicitly authorizes this commit. Never stage the prompt
suite, use broad staging, or create an empty commit.

## Final report additions

Report source evidence, parsed contract, unsupported cases, fixture coverage,
files, behavioral changes, preserved behavior, tests, validation, commit hash,
exact Git status, and the Task 05 gate.
