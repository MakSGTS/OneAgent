# Task 08: Complete Sprint 7 production evidence

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

- `docs/Roadmap.md`, Sprint 7 Task 08 and completion gates;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when Tasks 01–07 are committed in dependency order or individually
proven `already_complete`, all predecessor focused and workspace checks pass,
and no predecessor-created uncommitted change remains.

## Task

Close the smallest representative real-format production evidence matrix for
every accepted Sprint 7 capability. Transition Coverage only after the complete
evidence exists, recompute live aggregates, and synchronize current-state
documentation. This task does not decide Sprint completion.

## Production evidence scope

Use `FileSystemEdtSemanticGraphBuilder` over repository-owned EDT layouts, not
isolated helpers alone. Add the smallest fixture matrix necessary to prove
subordinate Form, subordinate Command, Common Command, and existing Common Form
module behavior; Common and subordinate Command parameter references; explicit
Common and subordinate Form navigation; and the accepted negative/partial
outcomes.

## Scope

Representative production evidence, independent Coverage transitions, live
aggregate verification, and current-state documentation.

## Included

- End-to-end Module, Procedure, Function, Query, ownership, reference,
  dependency, and `Opens` evidence required by ADR-0029.
- Positive, malformed, unsupported, missing, ambiguous, incompatible, partial,
  duplicate, equal-name, reordered-source, and repeated-build evidence.
- Deterministic node, edge, request, diagnostic, report, statistics, and
  provenance output.
- Query, Validation, graph/build Diff, reverse Impact, and complete/incremental
  Semantic Index equivalence with a clean rebuild.
- Independent graph-domain and EDT Coverage changes only for capabilities whose
  complete required evidence now passes.
- Representative test links, limitations, and aggregate counts derived from
  the live registries.
- Synchronize `docs/architecture/semantic-model-2.md` and `docs/Roadmap.md`
  with implemented and deferred scope without marking Sprint 7 completed.

## Excluded

- New architecture, target grammars, node/edge kinds, parser redesign, or
  semantics beyond Tasks 01–07.
- Coverage status inferred from ADR or isolated unit tests.
- Reopening or double-counting existing Form/Command declaration capabilities.
- Form internals, Command Groups, multilingual payload, dynamic/default/
  shorthand targets, explicit execution, placeholders, or Designer XML.
- Sprint 7 completion or Sprint 8 eligibility; Task 09 owns that decision.

## Acceptance criteria

- Every applicable ADR-0029 criterion maps to an exact meaningful test through
  the production builder or the required graph-domain consumer.
- No `Supported` capability has missing evidence or a contradictory
  limitation.
- Module ownership, Command requests/projections, and `Opens` production each
  have independent positive and negative evidence.
- Request ledgers, projections, diagnostics, reports, statistics, and Diff are
  mutually consistent without duplicate counting.
- Complete and incremental indexes are equivalent after relevant additions,
  modifications, removals, and clean rebuild.
- Registry counts and statuses are recomputed from live registries and verified
  by tests; no historical count is copied.
- Roadmap and Semantic Model describe current implementation and deferred scope
  truthfully while Sprint 7 remains incomplete pending review.
- If evidence reveals missing predecessor implementation, stop and report the
  failed predecessor criterion instead of hiding a semantic fix in Coverage
  work.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --test coverage
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-edt coverage
cargo test -p oneagent-edt
```

Confirm every filter runs meaningful tests. Then run the complete workspace
validation from `docs/codex/core/validation.md`, including
`git diff --check`.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. Stage only task-owned fixtures,
tests, Coverage registries, current-state documentation, and strictly necessary
evidence plumbing, then create one commit:

```text
Complete Sprint 7 production evidence
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report the production evidence matrix, Coverage transitions, aggregate
verification, remaining limitations, documentation sync, files, tests,
validation, commit hash, exact Git status, and the Task 09 gate.
