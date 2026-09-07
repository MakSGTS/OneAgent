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

# Review Sprint 41 Integration

## Reporting

Communicate in Russian; keep repository artifacts and commit messages in English.
Compose the selected Profile, base Task Template, specialized Template, and their
required Core/Workflow modules; permanent rules remain in those owners.

## Context manifest

### Must read

- `AGENTS.md` — repository boundary and branch/review workflow.
- `docs/Roadmap.md` — sections: Sprint 41 Safe Edit Transactions execution plan and Sprint efficiency contract within that plan.
- `docs/adr/0064-safe-edit-transactions.md` — Decision and Deferred scope.
- `docs/architecture/safe-edit-transactions-invariants.md` — section: Sprint 41 ADR invariant matrix.
- `docs/architecture/safe-edit-transactions-evidence.md` — final acceptance/validation and scope evidence.
- `docs/codex/workflows/review.md` — Independent sprint integration review and Completion-gate outputs.
- `docs/codex/prompts/sprint-41-safe-edit-transactions/00-sprint-41-execution-loop.md` — previous-suite inventory and manifest.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Document Sprint 41 Safe Edit Transaction Evidence` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean task-owned tree. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

## Task

Independent integration decision, Sprint 41 completion, and v0.7 release-review handoff.

## Scope

### Included

After Task 6, require the no-ff implementation merge into `codex/v0.7` and review branch `codex/v0.7-sprint-41-review`. Independently review the exact immutable range. After non-blocking reconciliation and same-reviewer artifact consistency, create `docs/reviews/sprint-41-safe-edit-transactions.md`, mark Sprint 41 completed, and retire exactly the four tracked Sprint 40.1 prompt files listed in the master. Preserve Sprint 40 and current Sprint 41 suites.

### Excluded

Other refactoring families; metadata/file/path renames; multi-Configuration or
cross-Workspace mutation; remote/Git mutation; new MCP/HTTP/CLI/LSP/IDE edit
surfaces; automatic model-generated edits; new production dependencies;
persisted cross-process plans/undo history; broad performance/security claims;
Sprint 42 and v0.7 release execution. Preserve unrelated user changes.

## Acceptance criteria

One fresh read-only reviewer and the primary independently inspect and validate the immutable implementation range against every accepted invariant. Keep reviewer identity/results separate from primary evidence. No fixes in this review. Obtain same-reviewer artifact consistency before state transition, explicit retirement, staging, and the single review commit. Any blocker preserves active state and all old prompts. A pass makes only the v0.7 release integration review eligible: do not start Sprint 42, release, tag, or merge to main. The dispatcher merges the successful review into codex/v0.7 and performs the user's end-of-sprint push.

## Task-specific validation

Independent reviewer and primary focused/full gates, artifact consistency, exact retirement audit. Use `docs/codex/core/validation.md` as the canonical matrix;
report zero matches separately. Validate this suite explicitly when prompts or
its efficiency records change. Keep large logs under
`local-artifacts/codex-runs/sprint-41/` and return only compact exact evidence.

## Review target

The complete Sprint 41 accepted implementation and compatibility boundary.

## Reviewed baseline / commit or diff range

Base ceb3a91da70afde202cab84f7ea42846cdd734bd; end is the full immutable no-ff implementation merge ID supplied by the dispatcher. Review on codex/v0.7-sprint-41-review with a clean tree.

## Review Criteria

Every ADR invariant, negative production oracle, actual scope/budget, authorization, freshness, confinement, rollback, undo, semantic publication, and preserved consumer contract.

## Acceptance evidence matrix

Reconcile the committed production matrix and final exact-head evidence; do not reuse historical counts.

## Independent reviewer contract and output

The dispatcher coordinates a separate fresh read-only reviewer under the Review workflow. This child does not delegate. Preserve the same reviewer for artifact consistency.

## Automatic independent-reviewer authorization

Resolve from the current user instruction and higher-priority runtime rules before dispatch.

## Primary/reviewer evidence reconciliation

Follow the Review workflow with independently executed focused/full validation and a decision no less severe than the reviewer recommendation.

## Authorized review outputs and state transition

Only the named review artifact, Sprint 41 completion/release-review handoff, and exact conditional four-file retirement; no source fix or release execution.

## Suggested commit message

```text
Complete Sprint 41 Safe Edit Transactions Review
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.
