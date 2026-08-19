# Task 05: Integrate Command parameter references

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

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 7 Task 05;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when Task 04 is committed with message
`Parse Sprint 7 command parameter references` or is proven
`already_complete`, and its observations implement the exact accepted
Command source and target contract.

## Task

Convert only Task 04 accepted observations into the public reference-request
lifecycle and deterministic terminal `References` and justified
`DependsOn` projections.

## Source contract

Translate source-specific Command observations at collection time without
leaking EDT types into `oneagent-graph`. Reuse the live public request,
ledger, resolution, report, build-validation, statistics, and build-Diff APIs.
Do not invent a new public category when the accepted contract can be
represented by the existing source-independent vocabulary.

## Scope

Request collection, resolution, terminal projections, and build-level
observability for the Task 04 source family.

## Included

- Collection-time requests for canonical Common and subordinate Command sources
  with immutable identity and deterministic collection provenance.
- Exact name-and-kind resolution against only the accepted nine target kinds,
  with explicit workspace completeness.
- Sorted expected kinds, candidates, provenance aggregation, ledger ordering,
  and source-order-independent duplicate handling.
- Resolved, missing, ambiguous, incompatible, partial, invalid, malformed,
  unsupported, and duplicate terminal behavior required by ADR-0029.
- One direct `References` and one derived `DependsOn` projection only for
  unique accepted resolution.
- Typed diagnostics, statistics derived once, reports, build validation,
  graph/build Diff, Query, Impact, and repeated-build evidence.

## Excluded

- New target families, Defined Types, primitive/platform targets, wildcard
  metadata endpoints, placeholders, Unknown nodes, or lower-confidence edges.
- Command execution, module ownership, `OpenForm`, or `Opens` behavior.
- Changes to metadata-member or access-right request semantics and Coverage.
- Double-counting one request through edge and diagnostic projections.
- Coverage transitions or Sprint status changes.

## Acceptance criteria

- Every accepted source observation becomes one canonical request with
  collection provenance before resolution.
- Request identity is stable across terminal states and explicit partial to
  complete workspace transitions.
- Duplicate observations aggregate provenance without duplicate requests,
  projections, diagnostics, or statistics.
- Resolved requests have exactly one compatible candidate and emit the exact
  two accepted edges from the canonical Command source.
- Every non-resolved outcome emits no resolved edge and no placeholder.
- Reports, diagnostics, ledger queries, statistics, build validation, and Diff
  remain internally consistent and deterministic.
- Existing metadata-member and access-right request pipelines remain unchanged
  and green.
- Reversed equivalent source order and repeated builds are equal.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test build_diff
cargo test -p oneagent-graph --test report
cargo test -p oneagent-edt
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. Stage only task-owned graph/EDT
implementation, fixture, test, and necessary documentation paths, then create
one commit:

```text
Integrate Sprint 7 command references
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report the public request mapping, identity and provenance lifecycle, resolution
and projection behavior, diagnostics/statistics/report impact, preserved
request families, files, tests, validation, commit hash, exact Git status, and
the Task 06 gate.
