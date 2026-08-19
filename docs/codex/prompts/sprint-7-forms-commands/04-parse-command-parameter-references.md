# Task 04: Parse Command parameter references

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

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 7 Task 04;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when Task 03 is committed with message
`Emit Sprint 7 form and command modules` or is proven `already_complete`.
Reinspect live Common and subordinate Command descriptors and current metadata
reader APIs before changing parser output.

## Task

Preserve direct `commandParameterType/types` values from Common and subordinate
Commands as typed deterministic parser observations. Do not resolve targets,
create public requests, or emit graph facts in this task.

## Source evidence

Use repository-owned Common Command and subordinate Command artifacts recorded
by the source investigation. Locate real Catalog, Document, Task, empty,
multiple, and deferred Defined Type examples with `rg` before writing tests.
Start from the live metadata-object and metadata-structure readers,
`EdtMetadataReferenceRole`, descriptor types, and their consumers.

## Scope

Command-owned parameter-type source observations only.

## Included

- Preserve canonical Common or subordinate Command source identity.
- Add or reuse a distinct semantic role equivalent to
  `CommandParameterType`; do not report it as an Attribute/member type.
- Preserve the raw token, mapped target kind, and canonical target name.
- Accept only Catalog, Document, Enumeration, Information Register,
  Accumulation Register, Accounting Register, Calculation Register, Business
  Process, and Task reference mappings.
- Define deterministic accepted, ignored, unsupported, malformed, duplicate,
  missing, and multiple-value behavior.
- Add real-format and generated positive, negative, reordering, duplicate, and
  repeated-read tests for both Command source forms.

## Excluded

- Target resolution, `SemanticReferenceRequest`, graph insertion, diagnostics
  projection, statistics, or Coverage.
- Primitive, Defined Type, platform, unknown, or new metadata mappings.
- Inferring parameter types from BSL or Command module signatures.
- Broadening Attribute, Dimension, or Resource type parsing.
- Command payload, group, representation, localization, or Form behavior.

## Acceptance criteria

- Each accepted value is tied to one canonical Command source and one distinct
  Command-parameter role.
- The nine-kind allowlist is exact; unsupported prefixes never degrade to a
  lower-confidence accepted mapping.
- Empty, missing, malformed, duplicate, and multiple values follow explicit
  deterministic typed policies.
- Equivalent reordered descriptors and repeated reads return equal ordered
  observations.
- Existing member-type observations and Common/subordinate Command declaration
  parsing remain compatible.
- The parser emits no request, graph node, edge, projected diagnostic, or
  Coverage change.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-edt metadata_structure
cargo test -p oneagent-edt metadata_object
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report any
zero-match filter rather than treating it as evidence.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. Stage only task-owned parser,
fixture, test, and necessary documentation paths, then create one commit:

```text
Parse Sprint 7 command parameter references
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report source artifacts, parser contract, role and allowlist, typed negative
outcomes, fixture coverage, preserved parsing behavior, files, tests,
validation, commit hash, exact Git status, and the Task 05 gate.
