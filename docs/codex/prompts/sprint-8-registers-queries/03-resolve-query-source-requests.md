# Task 03: Resolve Query source requests

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/implementation.md`

## Template

`docs/codex/templates/implementation-task.md`

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 8 Task 03;
- `docs/architecture/register-query-source-investigation.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`.

## Required gate

Proceed only when Task 02 is committed with message
`Parse Sprint 8 direct register query sources` or current committed evidence
proves every Task 02 criterion `already_complete`.

## Task

Convert accepted parsed Query source occurrences into the existing public
`SemanticReferenceCategory::QuerySource` lifecycle and adapt deterministic EDT
resolution to produce terminal requests. Do not change production edge or
diagnostic projections in this task.

## Scope

One coherent collection-and-resolution implementation using existing public
request types and current private query-source resolution policy.

## Included

- Create collected QuerySource requests from an existing Query node, typed
  target reference, exactly one expected metadata NodeKind, raw occurrence and
  location context, and collection provenance.
- Preserve stable ADR-0024 identity based on source, category, reference, and
  sorted expected kinds; exclude state, candidates, occurrence order, and
  provenance from identity.
- Adapt the resolver for Catalog, Information Register, Accumulation Register,
  and Accounting Register exact kinds.
- Preserve locale-independent lowercase lookup without NFC/NFKC, compatible
  ambiguity precedence, deterministic candidates, and explicit complete versus
  partial workspace behavior.
- Produce terminal Resolved, MissingTarget, PartialWorkspace,
  AmbiguousTarget, and IncompatibleTargetKind requests with resolver
  provenance.
- Aggregate equivalent observations deterministically and reject conflicting
  terminal content as an invariant failure.
- Add focused collected-to-terminal, duplicate, reordered, and repeated
  resolution tests.

## Excluded

- Changes to production `Reads`, new `DependsOn`, diagnostics, statistics,
  reports, builder request exposure, Coverage, or Roadmap status.
- Parser grammar or category changes, graph endpoint changes, placeholder
  targets, public request categories/states, shared public resolution indexes,
  serialization, or persistence.
- Calculation Registers, virtual tables, and write target requests.

## Acceptance criteria

- One parsed occurrence maps to one canonical collected request with non-empty
  collection provenance and the exact Query source node.
- Every accepted category maps to exactly one expected metadata kind.
- Terminal request state, outcome, candidate count, and provenance satisfy
  existing graph-domain invariants.
- Missing and partial absence remain distinct and require explicit workspace
  scope.
- Multiple compatible candidates are ambiguous; one compatible candidate wins
  over incompatible same-name candidates; only incompatible candidates produce
  the incompatible outcome.
- Candidate and provenance ordering is stable across graph insertion order and
  repeated resolution.
- Existing private resolver behavior remains compatible until Task 04 changes
  production projections.
- No graph edge, diagnostic total, report, Coverage status, or public API beyond
  accepted existing request usage changes.

## Repository Safety

- Recheck Git state, request APIs, resolver definitions, all consumers, tests,
  and applicable `AGENTS.md` before editing.
- Preserve unrelated work and avoid broad refactors of other request families.
- Do not stage or commit without explicit launching authorization.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph reference_request
cargo test -p oneagent-edt query_source_resolution
```

Confirm both filters execute meaningful tests, then run the complete workspace
validation from `docs/codex/core/validation.md`, including `git diff --check`.

## Commit

When explicitly authorized, stage only task-owned request adaptation, resolver,
tests, and necessary documentation, then create one commit:

```text
Resolve Sprint 8 query source requests
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report request identity, target representation, collection/resolver provenance,
terminal outcomes, normalization and collision behavior, public API impact,
preserved production projections, files, tests, validation, commit hash, final
Git status, and the Task 04 gate.
