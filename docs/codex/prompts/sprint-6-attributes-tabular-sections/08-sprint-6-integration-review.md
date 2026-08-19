# Task 08: Review the integrated Sprint 6 baseline

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

Read the profile, template, all required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/attribute-tabular-section-source-investigation.md`
- the accepted Sprint 6 ADR created by Task 02;
- `docs/adr/0003-semantic-domain-model.md`
- `docs/adr/0006-semantic-graph.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0023-typed-metadata-payload.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0025-references-endpoint-validation.md`

## Required gate

Proceed only when Task 07 is committed with message
`Complete Sprint 6 member coverage`, all Tasks 01–07 are present in dependency
order or explicitly recorded as `already_complete`, and the working tree has no
task-created changes apart from the preserved prompt suite.

Before creating the authorized review record or Roadmap transition, print the
complete Change Contract from `docs/codex/core/change-contract.md` with exact
paths. Preserve the pre-existing prompt-suite baseline.

## Review target

Review the committed Sprint 6 range beginning after
`Plan Sprint 6 attributes and tabular sections` and ending at the Task 07
commit. Resolve the exact hashes from live history.

Create:

`docs/reviews/sprint-6-attributes-tabular-sections.md`

This task explicitly authorizes the review record and a Roadmap status/evidence
update only when the completion decision is non-blocking. Do not fix production
findings in the review task.

## Scope

- Task 01 source evidence and decision readiness.
- Task 02 ADR conformance and rejected/deferred scope.
- Task 03 source-independent model and public compatibility.
- Task 04 parser evidence and typed negative behavior.
- Task 05 identity, immediate ownership, provenance, and graph emission.
- Task 06 accepted references, request lifecycle, projections, diagnostics,
  statistics, and endpoint precision when applicable.
- Task 07 real production evidence, Coverage, aggregate counts, and docs.
- Graph Validation, Query, Resolution, Diff, Impact, reports, determinism, and
  repeated builds across the integrated baseline.
- Scope containment and regressions for completed Sprint 3 behavior.

## Excluded

- Production fixes, refactors, new tests, architecture redesign, or new scope.
- Forms, commands, Sprint 7 implementation, queries, roles, subsystems, event
  subscriptions, Designer XML, Runtime, persistence, AI, MCP, and IDE work.
- Marking v0.3 complete; its release review follows Sprint 14.

## Review criteria

- Every accepted identity, ownership, content, reference, provenance,
  invalid-state, ordering, and repeated-build rule has executed evidence.
- Unknown source forms remain explicit rather than guessed.
- Nested members have exactly the accepted nearest owner and no false companion
  containment.
- No unresolved, ambiguous, partial, incompatible, Unknown, or placeholder
  target is emitted as a resolved relation.
- Public API compatibility and every affected consumer match the accepted ADR.
- Coverage reflects production evidence only and registry counts are current.
- Roadmap and Semantic Model 2 match the implementation and deferred scope.
- No excluded Sprint 7 or later concern was pulled forward.
- The full workspace validation succeeds at the reviewed Task 07 baseline.

## Acceptance evidence matrix

The review record must map every criterion to exact commits, files, tests, and
command results. Separate confirmed defects, missing evidence, open questions,
and non-blocking follow-ups. Issue one decision:

- `pass`;
- `pass with non-blocking follow-ups`;
- `blocked`.

Update Sprint 6 to `completed`, add the review link to completed-sprint evidence,
and make Sprint 7 the next planning target only for a non-blocking decision. A
`blocked` decision leaves Sprint 6 incomplete.

## Task-specific validation

Run the complete focused matrix from the accepted Sprint 6 ADR and Tasks 03–07,
including at minimum:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt
cargo test -p oneagent-edt --test ownership
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Record exact test
counts and report zero-match filters separately.

## Commit

After the review record, Roadmap transition when allowed, and all review
validation succeed, stage only the review record and explicit current-state
documentation paths. Create one commit:

```text
Complete Sprint 6 attributes and tabular sections review
```

The current user explicitly authorizes this commit even when a successfully
completed review records `blocked`; in that case do not mark Sprint 6 complete.
Never stage the prompt suite, use broad staging, or fix findings in this commit.

## Final report additions

Report the reviewed commit range, acceptance matrix, findings by severity,
missing evidence, scope conformance, decision, deferred work, exact validation,
files, commit hash, final HEAD, exact Git status, and whether Sprint 7 is now
eligible.
