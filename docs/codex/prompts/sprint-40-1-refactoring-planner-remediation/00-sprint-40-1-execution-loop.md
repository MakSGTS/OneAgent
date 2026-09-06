# Sprint 40.1 Refactoring Planner Remediation Execution Loop

Execute Sprint 40.1 strictly through
`docs/codex/workflows/sequential-sprint-execution.md`, opting this corrective
sprint into the Sprint 41+ efficiency and review controls.

## Reporting

- Communicate with the user in Russian.
- Keep repository artifacts, code, identifiers, errors, documentation, prompts,
  and commit messages in English.
- Report measured telemetry only; when unavailable, write `unavailable`.

## Canonical authorities

- `AGENTS.md`
- `docs/adr/0063-refactoring-planner.md`
- `docs/Roadmap.md` — sections: Sprint 40.1 Refactoring Planner Remediation
  execution plan and Sprint 40.1 ADR invariant matrix
- `docs/codex/core/validation.md`
- `docs/codex/workflows/sequential-sprint-execution.md`
- `docs/codex/workflows/review.md`
- each child prompt and its selected Prompt Contract v2 modules

## Sprint objective and state

Correct the two confirmed Sprint 40 completion blockers: make
`oneagent-analysis::publication::WorkspacePublicationId` the canonical checked
publication identity while preserving `ChangeImpactPublicationId` as a
source-compatible alias, and synchronize the final evidence with the live
post-remediation test inventory. Audit the six governance-only efficiency
commits separately from the product implementation. Sprint 40 is
administratively `completed` without a passing integration review, and Sprint
40.1 is `next` until Task 1 starts.

## Starting-state requirements

- Branch: `codex/v0.7-sprint-40.1` from version head `427a78cd`.
- The committed planning baseline contains this complete suite and the matching
  Roadmap plan and invariant matrix.
- The task-owned worktree is clean and every child passes
  `scripts/validate-codex-prompts.sh`.
- Commit mode is resolved only from the current launch instruction. The current
  launch authorizes one commit per completed task, immediate push, required
  no-ff merges, and sprint closure only after a non-blocking final review.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Outcome | Validation additions | Commit message |
|---:|---|---|---|---|---|
| 1 | `01-review-refactoring-planner-remediation-design.md` | Sprint 40.1 planning baseline | Fresh-context targeted design decision over the accepted publication-owner correction and invariant matrix | Exact planning range, owner/alias/consumer matrix, negative oracle, scope baseline, and documentation checks | `Approve Sprint 40.1 remediation design` |
| 2 | `02-remediate-refactoring-planner-contract.md` | Task 1 committed `pass` artifact | Canonical Workspace publication identity, source-compatible Change Impact alias, Runtime migration, regressions, and synchronized evidence | Non-zero Analysis/Runtime/adapters suites, inventory, public API and scope audits, one stable full gate | `Remediate Sprint 40.1 Refactoring Planner contract` |
| 3 | `03-sprint-40-1-integration-review.md` | Task 2 commit, successful validation, no-ff implementation merge into `codex/v0.7`, and immutable review branch | Independent review, primary reconciliation, artifact consistency, Sprint 40.1 closure, Sprint 41 hand-off, and conditional Sprint 39 suite retirement | Exact immutable combined and corrective ranges, complete matrices, scope classification, links, inventories, and cleanliness | `Complete Sprint 40.1 Refactoring Planner remediation review` |

All child prompts use `prompt_contract: v2`, `fresh_context: required`, bounded
Context Manifests, and validated framework selectors.

## Sprint efficiency contract

sprint_efficiency_contract: v1
adr_invariant_matrix: docs/Roadmap.md::Sprint 40.1 ADR invariant matrix
design_review_gate: docs/codex/prompts/sprint-40-1-refactoring-planner-remediation/01-review-refactoring-planner-remediation-design.md|Sprint 40.1 planning baseline|docs/reviews/sprint-40-1-refactoring-planner-remediation-design.md|Approve Sprint 40.1 remediation design
implementation_baseline: docs/codex/prompts/sprint-40-1-refactoring-planner-remediation/02-remediate-refactoring-planner-contract.md|10|600|none|9|1

## Previous-suite inventory

Only Task 3 may conditionally retire the exact eight tracked files under
`docs/codex/prompts/sprint-39-change-impact-analysis/` listed by the Roadmap.
Sprint 40 and Sprint 40.1 prompt suites must be preserved as the implementation
and corrective execution record.

## Dispatch and ledger additions

- Resolve exact commit IDs, status, prerequisite, scope baseline, and branch
  before each task.
- Start every child in a guaranteed fresh context and prohibit delegation.
- For Task 1, start exactly one fresh-context read-only design reviewer. Commit
  the predeclared artifact only after a `pass` decision.
- For Task 2, calculate task-owned paths and text churn from its exact starting
  commit and enforce the committed stop-loss baseline before the full gate.
- After Task 2, merge the implementation branch into `codex/v0.7` with
  `--no-ff`, push, and create `codex/v0.7-sprint-40.1-review` from that exact
  version head.
- For Task 3, start exactly one new fresh-context read-only integration reviewer
  against immutable commit endpoints. Do not mutate reviewed paths while it
  runs.
- Verify each commit and immediate push before proceeding.

## Failure and review gates

Stop after the first context, prerequisite, design-review, scope, validation,
commit, push, integration-review, reconciliation, artifact-consistency, or
retirement-inventory failure. Do not close Sprint 40.1 on a blocking or
incomplete result.

Task 3 may transition Sprint 40.1 to `completed` and Sprint 41 to `next` only
after the independent and primary matrices pass and the same reviewer confirms
the complete uncommitted review artifact and state diff.

## Final report additions

- Report the ordered ledger, exact branches and commit IDs, pushes and merges.
- Report design and integration reviewer identities, freshness/read-only proof,
  decisions, reconciliation, and artifact consistency.
- Report planned versus actual paths/churn, exact test inventory, retained and
  retired prompts, final state, and next action.
