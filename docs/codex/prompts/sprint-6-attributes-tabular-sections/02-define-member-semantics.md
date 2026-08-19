# Task 02: Define Sprint 6 member semantics

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep repository code, identifiers, documentation, tests, errors, public APIs,
  prompt text, and commit messages in English.
- Report only live evidence or accepted architecture.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

Read the profile, template, all required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/attribute-tabular-section-source-investigation.md`
- `docs/adr/0003-semantic-domain-model.md`
- `docs/adr/0006-semantic-graph.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0023-typed-metadata-payload.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0025-references-endpoint-validation.md`

## Required gate

Proceed only when Task 01 is committed with message
`Investigate Sprint 6 member source contracts`, its investigation record is
complete, and it identifies a decision-ready additional Sprint 6 source slice.
If evidence remains insufficient, make no changes and report the exact missing
artifact or contract.

## Task

Accept the smallest source-independent Attribute and TabularSection semantic
contract justified by Task 01. Create the next available ADR; at prompt
preparation time the intended path was:

`docs/adr/0028-attribute-tabular-section-semantics.md`

Verify the ADR number is still available before editing. If another accepted
ADR has taken that number, use the next available number and report the final
path. Update only the minimum Roadmap references required to make the accepted
contract and implementation order discoverable.

This is architecture-only work. Do not implement production behavior or change
Coverage status.

## Included

- Canonical member vocabulary and the exact accepted source-to-domain boundary.
- Identity inputs, UUID preservation, owner-scoped fallback encoding, collision
  and duplicate-name behavior.
- Immediate-owner and invalid-owner rules, including nearest-owner semantics.
- Accepted member content, equality, deterministic ordering, and semantic
  content versus identity.
- Reference categories, target allowlists, resolution states, projections, and
  statistics only when Task 01 supplies source evidence.
- Provenance ownership and source-context requirements.
- Validation, Query, Diff, Impact, report, serialization, public API, and
  compatibility impact.
- Typed malformed, unsupported, missing, ambiguous, partial, and duplicate
  outcomes.
- Coverage completion criteria and ordered prerequisites for Tasks 3–8.

## Excluded

- Rust implementation, fixtures, tests, or registry transitions.
- Reopening completed Sprint 3 behavior without contradictory evidence.
- Copying member nodes or relations into top-level metadata payload.
- New graph kinds, edges, reference mappings, tabular-section references, or
  deeper ownership forms without Task 01 evidence.
- Sprint 7 and later scope.
- Codex Framework changes unless the live readiness audit proves a concrete
  reusable gap.

## Acceptance criteria

- The ADR separates confirmed source evidence from the normative semantic
  decision.
- One minimal production slice is executable without guessing XML or public
  API behavior.
- Existing UUID identity, UUID-less owner scope, single immediate ownership,
  exact name-and-kind resolution, deterministic request ledgers, and stable
  Diff identity are preserved or explicitly migrated with consumer impact.
- Endpoint and ownership matrices are precise; unsupported cases emit no false
  nodes or edges.
- Public compatibility decisions precede implementation.
- Rejected alternatives, deferred scope, risks, implementation prerequisites,
  validation requirements, and Coverage gates are explicit.
- The Roadmap remains truthful and Sprint 6 is not marked completed.

## Task-specific validation

```bash
git diff --check
git diff -- docs/adr docs/Roadmap.md docs/codex
git status --short
```

Manually verify ADR numbering, links, status, and Roadmap consistency. Do not
run broad Rust validation for architecture-only work.

## Commit

After successful validation, stage only the new ADR and any explicitly required
Roadmap change, then create one commit:

```text
Define Sprint 6 member semantics
```

The current user explicitly authorizes this commit. Never stage the prompt
suite and never create an empty commit.

## Final report additions

Report the architecture decision, ADR path, rejected alternatives, deferred
scope, implementation prerequisites, files, validation, commit hash, exact Git
status, and the Task 03 gate.
