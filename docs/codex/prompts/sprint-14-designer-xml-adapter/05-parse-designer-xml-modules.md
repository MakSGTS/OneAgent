# Parse Sprint 14 Designer XML Modules

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
- `docs/architecture/semantic-model-2.md`
- accepted BSL declaration and graph ADRs referenced by ADR-0036

## Prerequisites / Required gate

Require committed Task 4 metadata descriptors and clean task-owned state.

## Task

Implement deterministic assembly and parsing of exactly the ADR-0036 accepted
Designer BSL module artifact layouts into source observations suitable for the
existing BSL analyzer, without graph emission.

## Source evidence / paired fixtures

Use exact paired Designer `Ext/*.bsl` and EDT module artifacts, preserving raw
hashes and documenting only ADR-0036 normalization used for comparison.

## Scope

### Included

- Exact accepted owner joins, module roles, paths, encodings/BOM handling,
  canonical source text, and source provenance.
- Missing optional module, duplicate role, orphan owner, wrong owner kind,
  ambiguous/mismatched path, unreadable, malformed encoding, reordered, and
  repeated cases applicable to the accepted slice.
- Independent module assembly/parser tests and BSL analyzer compatibility.

### Excluded

- Graph nodes/ownership/symbol contribution, forms/commands or other deferred
  module roles, cross-adapter conformance claims, and Coverage transitions.

## Acceptance Criteria

- Accepted modules join to exactly one canonical metadata/configuration owner
  and one stable module role independent of traversal order.
- Source normalization follows ADR-0036 and raw provenance remains available.
- Missing optional files do not create placeholders; structural conflicts are
  deterministic typed failures.
- The existing BSL analyzer accepts paired normalized content and exposes the
  same accepted declarations for representative evidence.
- Reordered and repeated reads are equal and EDT behavior is unchanged.

## Repository Safety

Modify only task-owned adapter/module/tests/fixture paths. Preserve graph
emission, EDT code, ignored corpora, `.codex/`, and suites.

## Task-specific Validation

- Focused new-adapter module tests and relevant `oneagent-bsl` tests, all with
  non-zero intended matches.
- Affected package checks and the complete workspace validation gate.

## Suggested commit message

`Parse Sprint 14 Designer XML modules`

## Final report additions

Report layouts, roles, owner joins, normalization/provenance, negative outcomes,
BSL compatibility, validation, commit, and Git state.
