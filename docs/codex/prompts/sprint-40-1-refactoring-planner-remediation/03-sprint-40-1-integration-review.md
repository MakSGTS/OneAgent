---
prompt_contract: v2
task_kind: review
profile: docs/codex/profiles/review.md
template: docs/codex/templates/review-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Review Sprint 40.1 Refactoring Planner Remediation

## Reporting

- Communicate with the user in Russian.
- Keep review artifacts, state documentation, and commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, validation, Git branch/release
  workflow, review/remediation branches, and GUI validation.
- `docs/adr/0063-refactoring-planner.md` — complete accepted Sprint 40 contract.
- `docs/Roadmap.md` — sections: Sprint 40 objective/evidence/gates, Sprint 40.1
  Refactoring Planner Remediation execution plan, and Sprint 40.1 ADR invariant
  matrix.
- `docs/architecture/refactoring-planner-evidence.md` — complete final evidence.
- `docs/reviews/sprint-40-1-refactoring-planner-remediation-design.md` — complete
  design-review decision.
- `docs/codex/workflows/review.md` and `docs/codex/core/validation.md` — complete
  current integration-review and validation contracts.
- Exact immutable combined and corrective ranges resolved by the prerequisite;
  inspect path/commit inventories before changed files.

### Lookup on demand

- Full changed source/test file and direct consumers — trigger: a diff hunk
  cannot prove an ADR-0063 or matrix invariant.
- Governance-only commits — trigger: product-scope classification is disputed.
- Retained logs — trigger: a required command fails or a count is inconsistent.

### Excluded from initial context

- implementation transcripts, design-review rationale, and the primary's
  expected decision;
- unrelated historical ranges, ADRs, reviews, and prompt suites;
- Sprint 41 transaction design beyond the exact hand-off and exclusions;
- complete fixtures, generated outputs, and successful logs.

### Preflight

- Reviewer and primary independently record effective window or `unknown`,
  measurement basis, admitted material, and `pass|warning|blocked`; narrow at
  warning and stop at the hard limit.

## Prerequisites / required gate

- Task 1 design artifact and Task 2 implementation/evidence commit are pushed.
- Task 2 passes focused checks and one canonical full gate within its baseline.
- The Sprint 40.1 implementation branch is merged `--no-ff` into
  `codex/v0.7`, pushed, and this task runs on
  `codex/v0.7-sprint-40.1-review` created from the exact version head.
- The combined baseline starts at completed Sprint 39 head
  `8d28ba8acacd00efd902eb2aa4ab3194f1636c05`; the corrective baseline starts at
  `427a78cd809a16bae2ee160867b20bb64c1d415e`; both endpoints are immutable.
- Verify the exact eight-file Sprint 39 retirement inventory live.

## Task

Perform the mandatory independent and primary completion review over the final
Sprint 40 plus Sprint 40.1 baseline. Prove that the confirmed publication-owner
and stale-evidence blockers are resolved, classify governance-only changes,
and re-evaluate every ADR-0063 acceptance criterion. For a non-blocking result,
create `docs/reviews/sprint-40-1-refactoring-planner-remediation.md`, transition
Sprint 40 and Sprint 40.1 to `completed`, make Sprint 41 the unique `next`
target, and retire exactly the verified Sprint 39 suite after same-reviewer
artifact consistency.

## Review target

- Combined product range:
  `8d28ba8acacd00efd902eb2aa4ab3194f1636c05..<Sprint 40.1 implementation merge>`.
- Corrective range:
  `427a78cd809a16bae2ee160867b20bb64c1d415e..<Sprint 40.1 implementation merge>`.
- Contract: ADR-0063, the committed Sprint 40.1 matrix, and explicit exclusions.

## Reviewed baseline / commit or diff range

- Record full hashes, parents, subjects, commit/path/churn counts, branch,
  initial/final status, implementation baseline actuals, and prompt inventories.

## Review Criteria

- Every ADR-0063 semantic/source/publication/plan/Workspace/cache/policy/
  protocol/client invariant and exclusion remains satisfied.
- `WorkspacePublicationId` owns the canonical newtype at the accepted public
  path; `ChangeImpactPublicationId` is type-identical compatibility only.
- One checked process-local sequence and all lifecycle/failure behavior remain.
- Final evidence counts reconcile exactly with independently enumerated tests.
- Governance-only efficiency changes are separated from product behavior and
  introduce no hidden product scope.
- No source mutation, edit authorization, transaction, rollback, unsupported
  family, sensitive leak, dependency/API break, or unrelated product change.

## Acceptance evidence matrix

- Map every ADR-0063 and Sprint 40.1 matrix criterion to separate reviewer and
  primary evidence, exact commands/counts, and `pass|blocked` status.

## Independent reviewer contract and output

- Launch exactly one guaranteed fresh-context read-only reviewer under
  `docs/codex/workflows/review.md` with only repository root, immutable ranges,
  authorities, criteria/exclusions, validation matrix, and output contract.
- Do not provide implementation reasoning, the design-review conclusion, the
  primary's expected decision, or proposed findings.

## Automatic independent-reviewer authorization

- The current Sprint 40.1 launch authorizes exactly this one integration
  reviewer; it may not delegate or mutate repository or Git state.

## Primary/reviewer evidence reconciliation

- The primary independently inspects both ranges and reruns the complete
  focused/canonical matrix after the reviewer. The effective decision may not
  be less severe; incomplete evidence or disagreement is `blocked`.

## Authorized review outputs and state transition

- Only after a non-blocking decision and complete validation, draft the review
  artifact plus exact README, Architecture, semantic-model, Roadmap, hand-off,
  and retirement diff.
- Ask the same reviewer to verify the complete uncommitted artifact and state
  diff read-only before deletion, staging, or commit.
- After consistency passes, delete only the eight explicitly listed Sprint 39
  files through the patch mechanism. Preserve Sprint 40, Sprint 40.1, and all
  other prompt suites.

## Scope

### Included

- Read-only integration review, independent/primary validation, reconciliation,
  review artifact, current-state transition, exact Sprint 39 retirement, one
  review commit, push, and no-ff review merge.

### Excluded

- Fixing findings, architecture changes, more reviewers, recursive deletion,
  Sprint 41 implementation, and release review.

## Acceptance criteria

- Reviewer freshness/read-only behavior and complete output are proven.
- Both immutable ranges and all matrix rows receive non-blocking evidence.
- Same-reviewer artifact consistency passes before any state change/deletion is
  staged or committed.
- Sprint 40 and Sprint 40.1 become `completed`, Sprint 41 becomes the unique
  `next` target, and prompt inventories agree.
- Review commit is pushed, merged `--no-ff` into `codex/v0.7`, and that merge is
  pushed with a clean final worktree.

## Task-specific validation

- Reviewer and primary each run the complete non-zero focused, compatibility,
  public-process, and canonical matrix from the synchronized evidence.
- Run test enumeration, range/API/dependency/scope/sensitive-data audits,
  Markdown links, prompt validation/inventories, `git diff --check`, and final
  cleanliness checks.

## Suggested commit message

`Complete Sprint 40.1 Refactoring Planner remediation review`

## Final report additions

- Report both reviewer gates, exact ranges, matrices, findings, missing
  evidence, reconciliation, effective decision, artifact consistency, state
  transition, prompts, commits/merges/pushes, and Sprint 41 next action.
