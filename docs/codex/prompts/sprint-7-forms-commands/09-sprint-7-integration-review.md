# Task 09: Review the integrated Sprint 7 baseline

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

- `docs/Roadmap.md`, Sprint 7 plan and completion gates;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0028-attribute-tabular-section-semantics.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when Task 08 is committed with message
`Complete Sprint 7 production evidence`, Tasks 01–08 are present in dependency
order or proven `already_complete`, every implementation validation gate
succeeded, and no task-created uncommitted change remains apart from the
preserved prompt suite.

## Review target and authorized outputs

Review the committed Sprint 7 range beginning after the accepted planning
baseline and ending at Task 08. Resolve exact hashes from live history.

Create:

`docs/reviews/sprint-7-forms-commands.md`

The task authorizes that review record and a bounded Roadmap state/evidence
transition. Update Sprint 7 to `completed` and make Sprint 8 eligible only for
a non-blocking decision with successful validation. Do not fix production
findings in this review change.

## Scope

- Task 01 graph model, precise endpoint matrices, consumers, indexes, Diff, and
  Impact policy.
- Task 02 module source parsing and typed negative behavior.
- Task 03 module identity, ownership, provenance, and existing BSL semantics.
- Task 04 Command parameter parser role and nine-kind boundary.
- Task 05 request lifecycle, resolution, projections, diagnostics, statistics,
  reports, and Diff.
- Task 06 exact static navigation extraction and rejected forms.
- Task 07 owner-scoped/Common Form resolution, `Opens`, Query, and Impact.
- Task 08 real production evidence, Coverage, aggregates, and current-state
  documentation.
- Compatibility for existing Form/Command identities, Common Form modules,
  metadata references, Calls, Reads, Writes, Includes, Grants, Extends, and
  Sprint 6 member behavior.

## Excluded

- Production fixes, refactors, new semantics, architecture redesign, or
  template/framework maintenance.
- Form internals, Command Groups, multilingual payload, dynamic/default/
  shorthand Forms, execution relations, placeholders, Designer XML, or later
  sprint work.
- Marking v0.3 complete; its release review follows Sprint 14.

## Review criteria

- Every accepted identity, ownership, reference, navigation, provenance,
  resolution, diagnostic, ordering, and repeated-build rule has executed
  evidence.
- `Opens` has only the exact accepted endpoints and no companion relation.
- Command parameter failures and navigation failures produce no false resolved
  edge or placeholder.
- Equal names remain owner-scoped, requests and edges deduplicate
  deterministically, and complete/incremental index results match.
- Public API and every exhaustive consumer match ADR-0029.
- Coverage reflects production evidence only and registry aggregates are live.
- Roadmap and Semantic Model match implementation and deferred scope.
- No excluded UI or later-sprint concern was pulled forward.
- Full workspace validation succeeds at the reviewed Task 08 baseline.

## Acceptance evidence matrix and decision

Map every criterion to exact commits, files, tests, and command results.
Separate confirmed defects, missing evidence, open questions, and non-blocking
follow-ups. Issue exactly one decision:

- `pass`;
- `pass with non-blocking follow-ups`;
- `blocked`.

A blocked decision leaves Sprint 7 incomplete and Sprint 8 ineligible. The
review record may still be committed when the review itself and its bounded
documentation validation succeed.

## Task-specific validation

Run the complete focused matrix from ADR-0029 and Tasks 01–08, including at
minimum:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-bsl
cargo test -p oneagent-graph
cargo test -p oneagent-edt
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Record exact
test counts and report zero-match filters separately.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. After the review record, allowed
Roadmap transition, and review validation succeed, stage only the review record
and explicit current-state documentation paths, then create one commit:

```text
Complete Sprint 7 forms and commands review
```

Never stage the prompt suite, use broad staging, fix findings, or create an
empty commit.

## Final report additions

Report the reviewed commit range, acceptance matrix, findings by severity,
missing evidence, scope conformance, decision, state transition, deferred work,
exact validation, files, commit hash, final HEAD, exact Git status, and whether
Sprint 8 and the post-sprint Framework audit are now eligible.
