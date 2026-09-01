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

# Review Sprint 40 Refactoring Planner

## Reporting

- Communicate with the user in Russian.
- Keep the review artifact, state documentation, and commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, validation, Git branch/release
  workflow, review/remediation branches, and GUI validation.
- `docs/adr/0063-refactoring-planner.md` — complete accepted Sprint 40 contract.
- `docs/architecture/refactoring-planner-evidence.md` — complete Task 9 evidence
  and validation hand-off.
- `docs/Roadmap.md` — sections: Sprint 40 objective, manifest, exclusions, state
  gates, and proposed Sprint 41 hand-off.
- `docs/codex/profiles/review.md`, `docs/codex/templates/review-task.md`, and
  `docs/codex/workflows/review.md` — complete selected independent-review
  contract.
- exact immutable implementation range — diff: completed Sprint 39 version head
  `8d28ba8a` through the Sprint 40 implementation merge on `codex/v0.7`;
  inspect path inventory before changed files.
- `docs/codex/core/validation.md` and the focused validation matrix named by
  ADR-0063/Task 9.

### Lookup on demand

- full changed source or test file — trigger: a diff hunk cannot establish an
  acceptance invariant; load that file and exact direct consumers.
- accepted prerequisite ADR — trigger: ADR-0063 cites an authority boundary
  disputed by the implementation; matching decision section only.
- client/host code and validation — trigger: the immutable range changes the
  corresponding source, schema, manifest, package, or workflow.
- retained logs — trigger: a required command fails or Task 9 reports a durable
  artifact necessary to reproduce evidence.

### Excluded from initial context

- implementation conversation transcripts and the primary's expected decision;
- unrelated historical ranges, ADRs, reviews, and prompt suites;
- complete fixture corpora, generated outputs, and successful logs;
- Sprint 41 design beyond exact hand-off and exclusions.

### Preflight

- Primary and reviewer independently record effective window or `unknown`,
  measurement basis, admitted material, and `pass|warning|blocked`.
- Narrow diff and consumer selectors at warning and stop at the hard limit.

## Prerequisites / required gate

- Tasks 1–9 and their pushes completed in dependency order.
- The sprint branch was merged with `git merge --no-ff` into `codex/v0.7`, the
  merge was pushed, and this task runs on
  `codex/v0.7-sprint-40-review` created from that version head.
- The exact immutable range, current branch/`HEAD`, status, validation matrix,
  and eight-file Sprint 39 retirement inventory are verified live.

## Task

Perform the mandatory independent and primary Sprint 40 integration review.
For a non-blocking effective decision, create
`docs/reviews/sprint-40-refactoring-planner.md`, transition Sprint 40 to
`completed` and Sprint 41 to `next` in current-state documents, and
conditionally retire exactly the verified Sprint 39 prompt suite after the
same reviewer passes artifact consistency.

## Review target

- Exact immutable range: `8d28ba8a..` the Sprint 40 implementation merge on
  `codex/v0.7`.
- Product contract: ADR-0063 and the Sprint 40 Roadmap acceptance boundary.

## Reviewed baseline / commit or diff range

- Resolve and record full hashes, parents, subjects, path/commit counts, branch,
  initial/final status, and prompt inventories before reviewer dispatch.

## Review Criteria

- Semantic/source authority, first-slice scope, immutable preconditions, plan
  and operation identity/order, duplicates/conflicts, completeness, preview,
  bounds, failures, redaction, cancellation, and deterministic evidence.
- Workspace publication/configuration/source binding, lifecycle, cache/watcher/
  Git compatibility, Tool Policy, protocol schema/catalog, public-process and
  client compatibility.
- No source/repository mutation, transaction, atomicity/rollback claim,
  unsupported refactoring, hidden dependency/API break, sensitive leak, or
  unrelated scope.
- Exact validation, documentation consistency, and Sprint 41 hand-off.

## Acceptance evidence matrix

- Map every ADR-0063 and Roadmap criterion to separate reviewer and primary
  evidence, exact commands/counts, and `pass|blocked` status.

## Independent reviewer contract and output

- Automatically launch exactly one guaranteed fresh-context read-only reviewer
  under `docs/codex/workflows/review.md` with only the repository root, immutable
  range, authorities, criteria/exclusions, validation matrix, and required
  output contract.
- Do not send the primary's rationale, expected decision, findings, or Task 9
  conclusions.

## Automatic independent-reviewer authorization

- The current Sprint 40 launch instruction authorizes exactly this one reviewer.
  The reviewer may not delegate or mutate any repository or Git state.

## Primary/reviewer evidence reconciliation

- The primary independently inspects the same range and reruns the complete
  focused/canonical matrix.
- The effective decision may not be less severe than the reviewer recommendation;
  unresolved disagreement or incomplete evidence is `blocked`.

## Authorized review outputs and state transition

- Only after a non-blocking decision and complete validation, draft the review
  artifact and exact current-state transition.
- Ask the same reviewer to verify the complete uncommitted artifact, transition,
  hand-off, and retirement inventory read-only before staging or deletion.
- After consistency passes, delete only the eight explicitly listed tracked
  files under `docs/codex/prompts/sprint-39-change-impact-analysis/` using the
  normal patch mechanism. Preserve the Sprint 40 suite and every other prompt.

## Scope

### Included

- Read-only review, independent/primary validation, review artifact, authorized
  state transition, exact previous-suite retirement, and one review commit.

### Excluded

- Fixing findings, implementation changes, architecture reselection, additional
  reviewers, recursive deletion, and Sprint 41 implementation.

## Acceptance criteria

- Reviewer freshness/read-only behavior and complete output are proven.
- Independent and primary evidence reconcile to a non-blocking effective
  decision with all required checks successful.
- Same-reviewer artifact consistency passes before state change, deletion,
  staging, or commit.
- Review/state/hand-off/prompt inventories agree and Sprint 40 implementation is
  unchanged.

## Task-specific validation

- Reviewer and primary each run the complete non-zero focused, compatibility,
  public-process, and canonical matrix from Task 9 and ADR-0063.
- Run range diff checks, test enumeration, API/dependency/scope/sensitive-data
  audits, Markdown links, prompt inventories, and final cleanliness checks.

## Suggested commit message

`Complete Sprint 40 Refactoring Planner review`

## Final report additions

- Report reviewer identity, fresh/read-only proof, recommendation, findings,
  missing evidence, primary reconciliation, effective decision, consistency,
  state transition, retired/preserved prompts, and exact next action.
