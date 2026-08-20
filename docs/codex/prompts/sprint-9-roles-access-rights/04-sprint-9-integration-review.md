# Task 04: Review the integrated Sprint 9 baseline

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

- `docs/Roadmap.md`, Sprint 9 execution plan and Task 04;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0019-grants-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0031-conditional-grants-semantics.md`;
- the committed Sprint 9 prompt suite.

## Required gate

Proceed only when the accepted Sprint 9 planning baseline and Tasks 01–03 are
committed or proven `already_complete`, all required implementation validation
passed, and no task-created uncommitted change remains.

## Review target

Review the exact committed range from the parent of the Sprint 9 planning
commit through the committed Task 03 head. Do not rely on prompt claims or
commit subjects as implementation proof.

## Reviewed baseline / commit or diff range

Resolve exact full hashes from live Git history. Verify commit order, subjects,
owned paths, prerequisite chain, and final Task 03 repository state before
reviewing behavior.

## Scope

- Every ADR-0031 graph model, identity, payload, production mapping,
  provenance, consumer, determinism, Coverage, compatibility, and documentation
  criterion.
- Exact unconditional ID/name compatibility and conditional separation.
- Positive, negative, malformed, missing, ambiguous, incompatible, duplicate,
  reordered, repeated-build, and index-equivalence evidence.
- Query, Diff, Impact, reports, validation, complete and incremental Semantic
  Index behavior.
- Full workspace Definition of Done and repository safety.
- Accepted-versus-deferred scope and Sprint 10 eligibility.

## Excluded

- Silent fixes to implementation, tests, ADRs, or prompts.
- New architecture, condition evaluation, deny, inheritance, defaults,
  profiles, groups, users, effective authorization, resource families,
  persistence, runtime APIs, or Sprint 10 planning.
- Refactoring or cleanup unrelated to a review finding.

## Review criteria

- Architecture conformance and one authoritative conditional direct-grant
  meaning.
- Deterministic, collision-safe identity with byte-compatible unconditional
  behavior.
- Typed payload compatibility and public API migration impact.
- Exact production resolution, aggregation, provenance, diagnostics/statistics,
  validation, and non-placeholder behavior.
- Generic consumer and complete/incremental index equivalence.
- No Coverage status/count drift and no unsupported completion claim.
- Full deferred-scope and repository-safety conformance.
- No blocking correctness, regression, missing-evidence, or documentation
  finding.

## Acceptance evidence matrix

Record pass/fail evidence for: planning/commit chain; graph payload; identity;
wrong-kind/empty input; real conditional source; absent restriction; duplicate
and distinct cases; negative resolution/parser outcomes; provenance; Query;
Diff; Impact; reports; validation; complete index; incremental equivalence;
Coverage; documentation; unconditional and unrelated behavior; workspace gate;
and deferred scope.

## Authorized review outputs and state transition

When and only when the decision is `pass` or `pass with non-blocking
follow-ups`, create `docs/reviews/sprint-9-roles-access-rights.md`, update
`docs/Roadmap.md` to mark Sprint 9 `completed` and Sprint 10 `next`, and
synchronize the final hand-off statement. Do not change implementation files.

For `blocked`, create no completion transition or partial review commit. Report
the finding and leave Sprint 9 incomplete.

## Task-specific validation

Run focused review checks:

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt role_rights
cargo test -p oneagent-edt --test grants
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

Record exact results and treat zero matched filters as missing evidence.

## Suggested commit message

```text
Complete Sprint 9 roles and access rights review
```

When authorized and the decision is non-blocking, stage only the review artifact
and Roadmap/current-state documentation transition explicitly owned by this
task. Never create a review commit for a blocked decision.

## Final report additions

Report the exact reviewed range, acceptance matrix, findings by severity,
missing evidence, scope/exclusion conformance, validation, decision, review
artifact, state transition, commit hash, final Git status, and Sprint 10
eligibility.
