# Sprint 40 Refactoring Planner Execution Loop

Execute Sprint 40 strictly through
`docs/codex/workflows/sequential-sprint-execution.md`.

## Reporting

- Communicate with the user in Russian.
- Keep repository artifacts, code, identifiers, errors, documentation, prompts,
  and commit messages in English.
- Report measured telemetry only; when unavailable, write `unavailable`.

## Canonical authorities

- `AGENTS.md`
- `docs/Roadmap.md` — sections: Sprint 40 Refactoring Planner execution plan
- `docs/codex/core/validation.md`
- `docs/codex/workflows/sequential-sprint-execution.md`
- each child prompt and its selected Prompt Contract v2 modules

## Sprint objective and state

Produce one bounded deterministic read-only semantic refactoring plan and
preview over an immutable complete Workspace publication, with explicit target,
precondition, conflict, completeness, bound, failure, policy, and public-product
contracts. Do not mutate source, repository, Graph, Workspace, cache, or client
state. Sprint 40 is `next` during planning and becomes `active` only when Task 1
starts from the committed planning baseline.

## Starting-state requirements

- Branch: `codex/v0.7-sprint-40`.
- Framework prerequisite: `5c273da1` (`Establish refactoring and safe edits
  prompt contracts`).
- The committed planning baseline contains this complete suite and the matching
  Roadmap manifest.
- The task-owned worktree is clean and every child passes
  `scripts/validate-codex-prompts.sh`.
- Commit mode is resolved only from the current launch instruction. The launch
  instruction for this run authorizes one commit per successfully completed
  task.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Outcome | Validation additions | Commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-refactoring-planner.md` | Sprint 40 planning baseline | Decision-ready semantic/source/precondition/consumer/oracle evidence | Non-mutating source, symbol, fixture, test, and focused-oracle checks | `Investigate Sprint 40 Refactoring Planner` |
| 2 | `02-define-refactoring-planner.md` | Task 1 commit | Accepted ADR-0063 for the bounded read-only planner | Markdown, links, decision completeness, and `git diff --check` | `Define Sprint 40 Refactoring Planner` |
| 3 | `03-implement-immutable-source-evidence.md` | Task 2 commit | Immutable source-document and exact BSL occurrence contracts | Non-zero Common source-document and BSL exact-occurrence tests | `Implement Sprint 40 immutable source evidence` |
| 4 | `04-integrate-adapter-source-evidence.md` | Task 3 commit | Paired EDT/Designer immutable source evidence | Non-zero adapter capture, controlled-change, and paired-conformance tests | `Integrate Sprint 40 adapter source evidence` |
| 5 | `05-implement-refactoring-plan-domain.md` | Task 4 commit | Immutable typed plan domain and closed validation surface | Non-zero domain identity, ordering, conflict, bound, and error tests | `Implement Sprint 40 refactoring plan domain` |
| 6 | `06-implement-validated-refactoring-planner.md` | Task 5 commit | Deterministic Graph-backed plan evaluation and preview | Non-zero planner, precondition, conflict, completeness, and repetition tests | `Implement Sprint 40 validated refactoring planner` |
| 7 | `07-integrate-workspace-refactoring-plans.md` | Task 6 commit | Immutable Workspace publication composition | Non-zero Workspace snapshot, lifecycle, failure, and source-format tests | `Integrate Sprint 40 Workspace refactoring plans` |
| 8 | `08-integrate-product-refactoring-planning.md` | Task 7 commit | Accepted read-only product projection and process behavior | Non-zero protocol, Tool Policy, MCP, and public-process tests | `Integrate Sprint 40 product refactoring planning` |
| 9 | `09-complete-refactoring-planner-evidence.md` | Task 8 commit plus committed diff-hygiene recovery | Complete validation and synchronized current-state evidence | Full focused, compatibility, API, dependency, scope, and canonical gates | `Complete Sprint 40 Refactoring Planner evidence` |
| 10 | `10-sprint-40-integration-review.md` | Task 9 commit, no-ff implementation merge into `codex/v0.7`, and review branch | Independent review, reconciliation, state transition, and conditional retirement | Exact immutable range, independent and primary matrices, artifact consistency | `Complete Sprint 40 Refactoring Planner review` |

All child prompts use `prompt_contract: v2`, `fresh_context: required`, bounded
Context Manifests, and validated framework selectors.

Task 9 recovery follows the first preserved validation failure only through the
committed `Remediate Sprint 40 diff hygiene` prerequisite. Its fresh runner may
admit the four preserved Task 9 documentation paths named by its prompt, must
audit them from scratch, and must reject every other uncommitted path.

## Previous-suite inventory

Only Task 10 may conditionally retire this exact immediately preceding suite:

- `docs/codex/prompts/sprint-39-change-impact-analysis/00-sprint-39-execution-loop.md`
- `docs/codex/prompts/sprint-39-change-impact-analysis/01-investigate-change-impact-analysis.md`
- `docs/codex/prompts/sprint-39-change-impact-analysis/02-define-change-impact-analysis.md`
- `docs/codex/prompts/sprint-39-change-impact-analysis/03-implement-change-impact-report.md`
- `docs/codex/prompts/sprint-39-change-impact-analysis/04-integrate-workspace-impact-snapshots.md`
- `docs/codex/prompts/sprint-39-change-impact-analysis/05-integrate-product-impact-reporting.md`
- `docs/codex/prompts/sprint-39-change-impact-analysis/06-complete-change-impact-evidence.md`
- `docs/codex/prompts/sprint-39-change-impact-analysis/07-sprint-39-integration-review.md`

## Dispatch and ledger additions

- Resolve the exact planning and task commit hashes live before each dispatch.
- Start every child with no inherited conversation turns and prohibit child
  delegation.
- Preserve only the compact ledger required by the sequential workflow.
- Verify every committed path inventory and push before starting the next child.
- Treat `already_complete` as valid only with complete committed evidence and
  successful task validation; create no empty commit.
- After Task 9, run required validation, merge the sprint branch into
  `codex/v0.7` with `--no-ff`, push the merge, create
  `codex/v0.7-sprint-40-review` from that version head, and dispatch Task 10
  there. After a successful Task 10 commit and push, merge the review branch back
  into `codex/v0.7` with `--no-ff` and push the version branch.

## Failure and review gates

Stop after the first context, prerequisite, evidence, implementation,
validation, commit, push, independent-review, reconciliation, artifact-
consistency, or retirement-inventory failure. Do not start later tasks.

Task 10 automatically launches exactly one separately authorized fresh-context
read-only reviewer under `docs/codex/workflows/review.md`. A non-blocking
effective decision, complete independent and primary validation, and a passing
same-reviewer artifact-consistency check are required before Roadmap transition
or prompt retirement.

## Final report additions

- Report starting and ending state, branch, `HEAD`, and status.
- Report the ordered ledger once, including commits, pushes, validation,
  preflight decisions, token telemetry or `unavailable`, and retained logs.
- Report review identity, read-only/fresh evidence, reconciliation, consistency,
  current-suite preservation, previous-suite retirement, and next-sprint
  eligibility.
