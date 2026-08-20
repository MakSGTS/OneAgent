# Emit Sprint 14 Designer XML Semantics

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
- `docs/architecture/semantic-model-2.md`
- accepted graph, provenance, metadata, and BSL declaration ADRs referenced by
  ADR-0036

## Prerequisites / Required gate

Require committed Tasks 3-5 and clean task-owned state. Do not reopen ADR-0036.

## Task

Implement the production Designer XML semantic builder that orchestrates the
committed discovery, configuration, metadata, and module stages into exactly the
accepted canonical graph slice.

## Source evidence / paired fixtures

Use the committed real-source-derived parser fixtures. Production tests must
enter through the public Designer builder rather than parser-only helpers.

## Scope

### Included

- Canonical configuration and accepted metadata nodes with existing identities,
  names, payloads, and immediate configuration ownership.
- Accepted module nodes, existing stable owner/role identities, ownership, BSL
  Procedure/Function declarations, and deterministic exact provenance.
- Explicit complete/partial orchestration, diagnostics or fatal outcomes exactly
  as ADR-0036 defines; production reordering, repeated build, Query, Diff,
  report, and Validation evidence.

### Excluded

- New graph kinds/APIs, metadata members, reference requests, non-Contains
  relations, deferred module roles, final paired conformance fixture, Coverage
  transitions, and release state changes.

## Acceptance Criteria

- The production builder emits only accepted facts and no EDT-local or
  source-path identity.
- Canonical node and ownership identities match accepted existing conventions;
  every child has exactly one owner and exact provenance.
- Malformed required input yields no successful partial graph; explicit partial
  input follows ADR-0036 without placeholder facts.
- Query, Diff, report, and Validation expose accepted facts deterministically.
- Reordered and repeated builds are equal; existing EDT tests and semantics pass.

## Repository Safety

Modify only task-owned Designer adapter/emission/tests paths and any exact public
consumer migration proven necessary. Preserve `.codex/`, corpora, suites, and
unrelated EDT/graph behavior.

## Task-specific Validation

- Focused Designer production builder, BSL declaration, graph Query/Diff/report,
  and Validation tests with non-zero intended matches.
- The complete workspace validation gate.

## Suggested commit message

`Emit Sprint 14 Designer XML semantics`

## Final report additions

Report production stages, canonical facts, scope/failure handling, provenance,
consumer evidence, EDT compatibility, validation, commit, and Git state.
