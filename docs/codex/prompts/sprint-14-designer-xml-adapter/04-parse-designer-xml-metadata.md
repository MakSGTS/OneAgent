# Parse Sprint 14 Designer XML Metadata

Continue OneAgent development.

## Reporting

- Repository content and commit message: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/source-adapter-implementation.md`

## Template

`docs/codex/templates/source-adapter-task.md`

## Authoritative documents

- `docs/adr/0036-designer-xml-adapter.md`
- `docs/architecture/designer-xml-source-investigation.md`
- `docs/architecture/designer-xml-source-corpus.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0008-edt-metadata-object-reader.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

Require committed Task 3 discovery/configuration loading and clean task-owned
state.

## Task

Implement deterministic artifact enumeration, assembly, and parsing for exactly
the ADR-0036 accepted top-level Designer XML metadata kinds, returning canonical
source-independent descriptors without graph emission.

## Source evidence / paired fixtures

Use exact real `<Family>/<Name>.xml` shapes and paired EDT descriptors recorded
by Task 1. Fixtures must retain source paths, raw hashes, and reduction details.

## Scope

### Included

- Canonical path ordering and exact directory/file joins.
- Accepted root element, UUID, name, synonym/payload, kind, and source evidence.
- Complete/partial, missing, duplicate, mismatch, malformed, unsupported,
  unknown, unreadable, reordered, and repeated parsing behavior applicable to
  ADR-0036.
- Independent parser and assembly tests.

### Excluded

- Modules, metadata members, relations, graph nodes/edges, conformance claims,
  Coverage transitions, and deferred kinds or nested artifacts.

## Acceptance Criteria

- Every emitted descriptor maps to an accepted existing `MetadataKind`, stable
  source UUID, exact canonical name, accepted payload, and artifact provenance.
- Directory/file/name/root conflicts are deterministic typed failures rather
  than first-match selection.
- Unknown or deferred artifacts follow ADR-0036 without synthetic Unknown facts.
- Equivalent reordering and repeated parsing are equal; valid siblings follow
  the accepted failure scope.
- No EDT parser structure leaks into public source-independent domain APIs.

## Repository Safety

Modify only task-owned adapter/parser/tests/fixture paths. Preserve EDT behavior,
ignored corpora, `.codex/`, prompt suites, and unrelated files.

## Task-specific Validation

- Focused new-adapter metadata parser and assembly tests with non-zero matches.
- Affected package checks and the complete workspace validation gate.

## Suggested commit message

`Parse Sprint 14 Designer XML metadata`

## Final report additions

Report enumerated kinds, assembly keys, completeness/failure behavior, canonical
mapping, deferred artifacts, validation, commit, and Git state.
