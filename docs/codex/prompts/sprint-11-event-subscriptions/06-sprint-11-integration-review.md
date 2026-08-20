# Task 06: Review the integrated Sprint 11 baseline

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

- `docs/Roadmap.md`, Sprint 11 execution plan and Task 06;
- `docs/architecture/event-subscription-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0012-bsl-symbols-in-semantic-graph.md`;
- `docs/adr/0016-cross-module-bsl-call-resolution.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`;
- the committed Sprint 11 prompt suite.

## Required gate

Proceed only when the accepted Sprint 11 planning baseline and Tasks 01–05 are
committed or proven `already_complete`, all required implementation validation
passed, and no task-created uncommitted change remains.

## Review target

Review the exact committed range from the parent of the Sprint 11 planning
commit through the committed Task 05 head. Do not rely on prompt claims or
commit subjects as implementation proof.

## Reviewed baseline / commit or diff range

Resolve exact full hashes from live Git history. Verify commit order, subjects,
owned paths, prerequisite chain, and final Task 05 repository state before
reviewing behavior.

## Scope

- Every ADR-0033 metadata, payload, parser, resolution, endpoint, identity,
  ownership, provenance, diagnostic, statistics, determinism, Coverage,
  compatibility, and documentation criterion.
- Exact/family/overlapping/unsupported source selectors and their positive,
  missing, ambiguous, incompatible, malformed, duplicate, reordered, and
  repeated-build outcomes.
- Exact Common Module Module/Procedure ownership, exported and non-exported
  handlers, and Function/wrong-owner/missing/ambiguous failures.
- Direct configuration ownership, source/handler References, handler Triggers,
  and absence of placeholder or reverse facts.
- Query, Diff, dependency/Impact policy, reports, validation, request-ledger
  compatibility, complete and incremental Semantic Index behavior.
- Tracked fixture derivation/hashes, truthful Coverage counts, full workspace
  Definition of Done, repository safety, previous-suite inventory, and Sprint
  12 eligibility.

## Excluded

- Silent fixes to implementation, tests, ADRs, prompts, fixtures, Coverage, or
  documentation.
- New architecture, unsupported metadata families, public multi-target request
  lifecycle, Triggers dependency policy, runtime dispatch, event-specific Query,
  persistence, Designer XML, or Sprint 12 planning.
- Refactoring or cleanup unrelated to a review finding.
- Deletion of the current Sprint 11 suite, bootstrap prompt, older non-adjacent
  suite, untracked file, or any path outside the verified Sprint 10 suite.

## Review criteria

- Architecture conformance and one canonical meaning per stored relation.
- Stable UUID identity, typed event payload, and exact configuration ownership.
- Source parser fidelity and deterministic fatal/recoverable boundaries.
- Exact and family resolution completeness without treating valid multi-target
  selection as ambiguity or polluting the ADR-0024 ledger.
- Handler ownership and non-exported acceptance without misusing Calls policy.
- Precise References and Triggers endpoints, provenance, diagnostics,
  statistics, and no placeholders.
- Generic consumer and complete/incremental index equivalence.
- References-only dependency/Impact behavior for the handler path.
- Truthful Coverage evidence/counts and implemented current-state docs.
- Full deferred-scope and repository-safety conformance.
- No blocking correctness, regression, missing-evidence, or documentation
  finding.

## Acceptance evidence matrix

Record pass/fail evidence for: planning/commit chain; source corpus and tracked
fixture provenance; metadata/payload/public enum model; parser fields and
errors; exact/family/overlap/unsupported source resolution; handler ownership
and export policy; node/edge identity; ownership; References; Triggers;
provenance; diagnostics/statistics; request ledger; generic Query; Diff;
dependency/Impact policy; reports; validation; complete index; incremental
equivalence; Coverage; documentation; unrelated compatibility; workspace gate;
and deferred scope.

## Authorized review outputs and state transition

When and only when the decision is `pass` or `pass with non-blocking
follow-ups`, create `docs/reviews/sprint-11-event-subscriptions.md`, update
`docs/Roadmap.md` to mark Sprint 11 `completed` and Sprint 12 `next`, and
synchronize the final hand-off statement. Change
`docs/architecture/semantic-model-2.md` only if its already-implemented current-
state statement requires the final review decision. Do not change implementation
files.

For `blocked`, create no completion transition, prompt retirement, or partial
review commit. Report the finding and leave Sprint 11 incomplete.

## Verified previous-suite retirement

Only after a non-blocking decision and every focused/full validation command
succeeds, re-enumerate and compare the live tracked inventory with exactly:

```text
docs/codex/prompts/sprint-10-subsystems-composition/00-sprint-10-execution-loop.md
docs/codex/prompts/sprint-10-subsystems-composition/01-implement-subsystem-hierarchy-graph-rules.md
docs/codex/prompts/sprint-10-subsystems-composition/02-parse-nested-subsystem-hierarchy.md
docs/codex/prompts/sprint-10-subsystems-composition/03-emit-nested-subsystem-composition.md
docs/codex/prompts/sprint-10-subsystems-composition/04-complete-sprint-10-production-evidence.md
docs/codex/prompts/sprint-10-subsystems-composition/05-sprint-10-integration-review.md
```

If and only if the inventory matches, no untracked file is endangered, and no
current Sprint 11 link depends on these files, explicitly delete these six
tracked files through the normal patch mechanism. Do not use recursive deletion
or globs. Keep the complete Sprint 11 suite and
`docs/codex/prompts/run-next-sprint.md` untouched. Stage the exact deletions,
review artifact, Roadmap transition, and any explicitly required final Semantic
Model state together in the single review commit. Any mismatch blocks deletion
and the final review commit.

## Task-specific validation

Run focused review checks:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib event_subscription::tests
cargo test -p oneagent-edt --lib event_subscription_resolution::tests
cargo test -p oneagent-edt --test event_subscriptions
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
Complete Sprint 11 event subscriptions review
```

When authorized and the decision is non-blocking, stage only the review
artifact, Roadmap/current-state transition, and the six exact verified Sprint
10 prompt deletions. Never create a review commit for a blocked decision.

## Final report additions

Report the exact reviewed range, acceptance matrix, findings by severity,
missing evidence, scope/exclusion conformance, validation, decision, review
artifact, state transition, every retired path, retained Sprint 11 suite,
commit hash, final Git status, and Sprint 12 eligibility.
