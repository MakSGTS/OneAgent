# Task 06: Review the integrated Sprint 8 baseline

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 8 plan and completion gates;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/register-query-source-investigation.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0022-writes-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`.

## Required gate

Proceed only when Task 05 is committed with message
`Complete Sprint 8 production evidence`, Tasks 01–05 are present in dependency
order or proven `already_complete`, every implementation validation gate
succeeded, and no task-created uncommitted change remains apart from preserved
prompt-suite files.

## Task

### Review target and baseline

Resolve the exact accepted Sprint 8 planning baseline and Task 05 head from
live Git history. Review every implementation and evidence commit in that
range. Historical prompt hashes or claims are not proof.

## Authorized review outputs and state transition

Create:

`docs/reviews/sprint-8-registers-queries.md`

The task also authorizes a bounded Roadmap state/evidence transition. Update
Sprint 8 to `completed` and make Sprint 9 the next planning target only after a
non-blocking decision and successful required validation. Do not fix production
findings in the review change.

## Included

- Task 01 exact Reads and Query DependsOn graph matrices and consumers.
- Task 02 parser categories, source provenance, reductions, locations, and
  preserved all-or-nothing grammar.
- Task 03 QuerySource request identity, lifecycle, provenance, resolution,
  duplicates, and workspace scope.
- Task 04 terminal request projections, diagnostics, statistics, Reads,
  DependsOn, Query, Diff, Impact, reports, validation, and determinism.
- Task 05 full-builder fixture evidence, index equivalence, Coverage evidence,
  aggregate stability, and current-state documentation.
- Compatibility for Query identity/ownership, Catalog/Information Register
  Reads, Writes, metadata and Command references/dependencies, Calls, Opens,
  Includes, Grants, Extends, payload, and completed Sprint 7 behavior.

## Excluded

- Production fixes, refactors, new architecture, parser expansion, or Framework
  maintenance.
- Calculation Registers, virtual tables, JOIN, UNION, nesting, batches,
  temporary/external/parameter tables, new Query declaration sources, Query
  mutation, write-derived dependencies, register payload/members, Designer XML,
  persistence, Runtime, API transport, or later-sprint work.
- Marking v0.3 complete; its release review follows Sprint 14.

## Review criteria

- Every accepted source, identity, endpoint, request, provenance, projection,
  diagnostic, statistics, ordering, and repeated-build rule has executed
  production evidence.
- Reduced fixtures are traceable to exact real source and do not overclaim full
  complex-query grammar.
- Unsupported or incomplete programs produce no partial request projection,
  Reads, or DependsOn.
- Every resolved QuerySource request produces exactly one Reads and one derived
  DependsOn per unique Query-target pair, with distinct canonical identities
  and deterministic provenance.
- Request-ledger, diagnostic, statistics, report, graph/build Diff, validation,
  Query, Impact, and complete/incremental index results reconcile.
- Existing identities, public APIs, production facts, Coverage statuses, and
  exact aggregates remain compatible.
- No excluded virtual-table, Calculation Register, grammar, declaration-source,
  write-derived, register-payload, or later-sprint concern entered the range.
- Full workspace validation succeeds at the reviewed Task 05 baseline.

## Acceptance evidence matrix and decision

Map every criterion to exact commits, files, tests, fixtures, and command
results. Separate defects, missing evidence, open questions, compatibility
breaks, and non-blocking follow-ups. Issue exactly one decision:

- `pass`;
- `pass with non-blocking follow-ups`;
- `blocked`.

A blocked decision leaves Sprint 8 incomplete and Sprint 9 ineligible. The
review artifact may still be committed when the review itself and bounded
documentation validation succeed.

## Repository Safety

- Recheck Git state, exact review range, authorities, implementation, tests,
  fixtures, Coverage, and applicable `AGENTS.md` before reviewing.
- Preserve unrelated work. Do not modify production files or silently repair
  findings.
- Do not stage or commit without explicit launching authorization.

## Task-specific validation

Run the complete focused Sprint 8 matrix from Tasks 01–05, including at minimum:

```bash
cargo test -p oneagent-bsl
cargo test -p oneagent-graph
cargo test -p oneagent-edt
```

Run the exact Sprint 8 integration filter separately and confirm it matches
tests. Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Record exact test
counts and report zero-match filters separately.

## Commit

When explicitly authorized and after the review record, allowed Roadmap
transition, and review validation succeed, stage only the review record and
explicit current-state documentation paths, then create one commit:

```text
Complete Sprint 8 registers and queries review
```

Never stage the prompt suite, use broad staging, fix findings, or create an
empty commit.

## Final report additions

Report the reviewed commit range, acceptance matrix, findings by severity,
missing evidence, compatibility and scope conformance, decision, state
transition, deferred work, exact validation and test counts, files, commit hash,
final HEAD, exact Git status, and whether Sprint 9 is now eligible.
