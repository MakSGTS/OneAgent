# Task 01: Investigate Sprint 6 member source contracts

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep repository files, identifiers, documentation, tests, errors, public APIs,
  prompt text, and commit messages in English.
- Distinguish confirmed repository evidence, accepted decisions, and unknowns.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

Read the profile, template, their required Core modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0003-semantic-domain-model.md`
- `docs/adr/0006-semantic-graph.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0023-typed-metadata-payload.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0025-references-endpoint-validation.md`

## Required gate

Proceed only when committed Roadmap and review evidence show Sprint 5 and the
v0.2 release review completed and Sprint 6 is the live planning and execution
target. Otherwise make no changes and report the exact missing evidence.

## Historical preparation evidence

At prompt preparation HEAD `922ba70`, EDT production already represented
`Attribute` and `TabularSection`, immediate nested ownership, owner-scoped
UUID-less fallback identity, metadata-type reference requests, provenance,
Query, Validation, Coverage, and repeated-build evidence. Treat this as
historical context only and recheck every fact.

Before editing the authorized investigation record, print the complete Change
Contract from `docs/codex/core/change-contract.md` with exact paths and preserve
the pre-existing prompt-suite baseline.

## Investigation objective

Produce the decision-ready, repository-owned source and capability inventory
required before Sprint 6 selects any additional Attribute or TabularSection
semantics.

Create:

`docs/architecture/attribute-tabular-section-source-investigation.md`

This task authorizes that investigation record only. Do not implement behavior.

## Questions to answer

1. Which Attribute and TabularSection source shapes are proven by real
   repository-owned EDT artifacts, generated test sources, and parser tests?
2. Which metadata-owner families are proven for top-level attributes, tabular
   sections, and nested attributes?
3. What are the live identity rules for UUID-present and UUID-absent members,
   equal names, immediate owners, and reordered source observations?
4. Which member fields and type forms are parsed, retained, discarded,
   malformed, unsupported, or still unknown?
5. Which graph, Query, Resolution, Validation, Diff, Impact, Coverage, report,
   and request-ledger consumers depend on the current representation?
6. Which completed Sprint 3 behavior must remain a compatibility baseline?
7. What is the smallest additional Sprint 6 production slice supported by
   evidence? If no additional slice is decision-ready, what exact artifacts or
   fixture evidence are missing?
8. Does the existing Codex Framework remain sufficient for Tasks 2–8?

## Evidence scope

Inspect at minimum:

- `crates/metadata/`;
- `crates/graph/`, including node kinds, containment, reference requests,
  Validation, Query, Diff, Impact, and Coverage;
- `adapters/edt/src/metadata_structure.rs` and its consumers in
  `adapters/edt/src/lib.rs`;
- `adapters/edt/src/coverage.rs`;
- `adapters/edt/tests/ownership.rs` and its complete real-format fixture;
- nearby parser and production tests for UUID fallback, composite types,
  duplicates, malformed values, source ordering, and repeated builds;
- relevant history and accepted architecture.

Do not treat inline XML test strings as proof of wider real-world EDT variants
without classifying their evidence level. Do not use external undocumented XML
as semantic authority.

## Excluded

- Rust implementation, fixtures, tests, ADR decisions, or Coverage changes.
- Invented XML fields, ownership forms, reference families, graph kinds, or
  target mappings.
- Forms, commands, queries, roles, subsystems, event subscriptions, Designer
  XML, Runtime, persistence, AI, MCP, and IDE work.
- Changes under `docs/codex/`.

## Completion criteria

- The record contains exact artifact paths and an implemented-versus-missing
  matrix.
- Confirmed, accepted, and unknown statements are separate.
- Identity, ownership, member content, reference, provenance, ordering,
  malformed-input, consumer, and Coverage evidence are traced.
- Completed first slices are not presented as new Sprint 6 work.
- One smallest decision-ready candidate slice is identified, or the record
  explicitly concludes that Task 02 is blocked by named missing evidence.
- Framework readiness is recorded without speculative template changes.

## Task-specific validation

```bash
git diff --check
git diff -- docs/architecture/attribute-tabular-section-source-investigation.md
git status --short
```

Manually verify every repository-relative link and cited path. Do not run broad
Rust validation for this documentation-only investigation.

## Commit

After successful validation, stage only the investigation record and create one
commit:

```text
Investigate Sprint 6 member source contracts
```

The current user explicitly authorizes this commit. Never stage files under
`docs/codex/prompts/`; do not create an empty commit.

## Final report additions

Report confirmed findings, accepted constraints, unknowns, decision readiness,
created files, validation, commit hash, exact Git status, and the next gate.
