# Task 05: Review the integrated Sprint 12 baseline

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, tests, errors,
  public APIs, prompt text, review artifacts, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 12 execution plan and Task 05;
- `docs/architecture/report-data-composition-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`;
- `docs/adr/0034-report-data-composition-semantics.md`;
- the committed Sprint 12 prompt suite.

## Required gate

Proceed only when the accepted Sprint 12 planning baseline and Tasks 01–04 are
committed or proven `already_complete`, all required implementation validation
passed, and no task-created uncommitted change remains.

## Review target

Review the exact committed range from the parent of the Sprint 12 planning
commit through the committed Task 04 head. Do not rely on prompt claims or
commit subjects as implementation proof.

## Reviewed baseline / commit or diff range

Resolve exact full hashes from live Git history. Verify commit order, subjects,
owned paths, prerequisite chain, and final Task 04 repository state before
reviewing behavior.

## Scope

- Every ADR-0034 node, payload, identity, ownership, parser, failure/deferred,
  provenance, diagnostic, statistics, determinism, Coverage, compatibility,
  and documentation criterion.
- Exact main/non-main/empty Query/Object/Union source shapes plus positive,
  missing, duplicate, mismatched, unsupported, nested, folder, malformed,
  reordered, and repeated outcomes.
- Report discovery compatibility, Schema/DataSet/Field/Query node identity,
  immediate Contains ownership, and absence of placeholder/reverse facts.
- Metadata-owned Query identity/content and strict absence of partial
  QuerySource, Reads, DependsOn, References, candidate, or query-language
  diagnostic projections.
- Generic Query, Diff, Impact policy, reports, Validation, complete and
  incremental Semantic Index behavior.
- Tracked fixture derivation/hashes, truthful Coverage counts, full workspace
  Definition of Done, repository safety, previous-suite inventory, and Sprint
  13 eligibility.

## Excluded

- Silent fixes to implementation, tests, ADRs, prompts, fixtures, Coverage, or
  documentation.
- New architecture, nested Union identities, field folders, broader DCS query
  grammar, data-source relations, runtime composition, non-Report schemas,
  persistence, Designer XML, XDTO/service work, or Sprint 13 planning.
- Refactoring or cleanup unrelated to a review finding.
- Deletion of the current Sprint 12 suite, bootstrap prompt, older non-adjacent
  suite, untracked file, or any path outside the verified Sprint 11 suite.

## Review criteria

- Architecture conformance and one canonical meaning per node/payload/edge.
- Stable UUID and collision-safe owner-scoped identities independent from
  content, position, and traversal.
- Parser fidelity, exact descriptor/artifact join, and deterministic fatal
  versus deferred boundaries.
- Precise immediate ownership, payload compatibility, provenance,
  diagnostics/statistics, and no placeholders.
- Metadata-owned Query compatibility and all-or-nothing query-source exclusion.
- Generic consumer and complete/incremental index equivalence.
- Contains-only Impact exclusion and unchanged unrelated request/relation policy.
- Truthful Coverage evidence/counts and implemented current-state docs.
- Full deferred-scope and repository-safety conformance.
- No blocking correctness, regression, missing-evidence, or documentation
  finding.

## Acceptance evidence matrix

Record pass/fail evidence for: planning/commit chain; source corpus and tracked
fixture provenance; node/payload/public enum model; identity/collision behavior;
parser fields/artifact join/errors; main/non-main/empty and Query/Object/Union;
nested/folder/unsupported outcomes; ownership; provenance; diagnostics/
statistics; metadata-owned Query; query-source relation absence; generic Query;
Diff; Impact policy; reports; Validation; complete index; incremental
equivalence; Coverage; documentation; unrelated compatibility; workspace gate;
and deferred scope.

## Authorized review outputs and state transition

When and only when the decision is `pass` or `pass with non-blocking
follow-ups`, create `docs/reviews/sprint-12-skd-report-model.md`, update
`docs/Roadmap.md` to mark Sprint 12 `completed` and Sprint 13 `next`, and
synchronize the final hand-off statement. Change
`docs/architecture/semantic-model-2.md` only if its already-implemented current-
state statement requires the final review decision. Do not change
implementation files.

For `blocked`, create no completion transition, prompt retirement, or partial
review commit. Report the finding and leave Sprint 12 incomplete.

## Verified previous-suite retirement

Only after a non-blocking decision and every focused/full validation command
succeeds, re-enumerate and compare the live tracked inventory with exactly:

```text
docs/codex/prompts/sprint-11-event-subscriptions/00-sprint-11-execution-loop.md
docs/codex/prompts/sprint-11-event-subscriptions/01-implement-event-subscription-graph-model.md
docs/codex/prompts/sprint-11-event-subscriptions/02-parse-event-subscription-descriptors.md
docs/codex/prompts/sprint-11-event-subscriptions/03-resolve-event-subscription-targets.md
docs/codex/prompts/sprint-11-event-subscriptions/04-emit-event-subscription-semantics.md
docs/codex/prompts/sprint-11-event-subscriptions/05-complete-sprint-11-production-evidence.md
docs/codex/prompts/sprint-11-event-subscriptions/06-sprint-11-integration-review.md
```

If and only if the inventory matches, no untracked file is endangered, and no
current Sprint 12 link depends on these files, explicitly delete these seven
tracked files through the normal patch mechanism. Do not use recursive deletion
or globs. Keep the complete Sprint 12 suite and
`docs/codex/prompts/run-next-sprint.md` untouched. Stage the exact deletions,
review artifact, Roadmap transition, and any explicitly required final Semantic
Model state together in the single review commit. Any mismatch blocks deletion
and the final review commit.

## Task-specific validation

Run focused review checks:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib report_data_composition::tests
cargo test -p oneagent-edt --test report_data_composition
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the canonical complete workspace validation:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Record exact results and treat zero matched filters as missing evidence. After
the authorized state/deletion change, rerun `git diff --check` and manually
validate prompt/link inventories before committing.

## Suggested commit message

```text
Complete Sprint 12 SKD and report model review
```

When authorized and the decision is non-blocking, stage only the review
artifact, Roadmap/current-state transition, and the seven exact verified Sprint
11 prompt deletions. Never create a review commit for a blocked decision.

## Final report additions

Report the exact reviewed range, acceptance matrix, findings by severity,
missing evidence, scope/exclusion conformance, validation, decision, review
artifact, state transition, every retired path, retained Sprint 12 suite,
commit hash, final Git status, and Sprint 13 eligibility.
