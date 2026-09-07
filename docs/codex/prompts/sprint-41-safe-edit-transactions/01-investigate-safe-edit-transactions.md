---
prompt_contract: v2
task_kind: investigation
profile: docs/codex/profiles/investigation.md
template: docs/codex/templates/investigation-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Investigate Safe Edit Transactions

## Reporting

Communicate in Russian; keep repository artifacts and commit messages in English.
Compose the selected Profile, base Task Template, specialized Template, and their
required Core/Workflow modules; permanent rules remain in those owners.

## Context manifest

### Must read

- `AGENTS.md` — repository boundary and branch/review workflow.
- `docs/Roadmap.md` — sections: Sprint 41 Safe Edit Transactions execution plan and Sprint efficiency contract within that plan.
- `docs/reviews/sprint-40-1-refactoring-planner-remediation.md` — sections: Decision and Prompt retirement and hand-off.
- `docs/adr/0063-refactoring-planner.md` — sections: Authority, owner, and dependency direction; Publication and target identity; Request, preconditions, and plan identity; Workspace lifecycle and persistent cache; Deferred scope.
- `docs/architecture/refactoring-planner-evidence.md` — section: Paired source and planner oracle.
- `crates/analysis/src/refactoring.rs` — symbols: RefactoringPlan, RefactoringPreconditionSet, RefactoringOperation, SourceEvidenceSet.
- `apps/runtime/src/workspace/mod.rs` — symbols: WorkspaceService, WorkspaceSnapshotObserver, WorkspaceSnapshotBuilder, plan_refactoring.
- `crates/tool-policy/src/confirmation.rs` and `execution.rs` — symbols: ToolConfirmationChallenge, ToolConfirmation, execute_tool.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Plan Sprint 41 Safe Edit Transactions` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean task-owned tree. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

## Task

Repository-backed transaction readiness and boundary investigation.

## Scope

### Included

Create `docs/architecture/safe-edit-transactions-investigation.md`; update only the Sprint 41 Roadmap investigation/state subsection. Trace all publication writers, watcher/cache rebuild paths, planner consumers, policy entry points, confinement primitives, and paired fixture tests before choosing a mechanism.

### Excluded

Other refactoring families; metadata/file/path renames; multi-Configuration or
cross-Workspace mutation; remote/Git mutation; new MCP/HTTP/CLI/LSP/IDE edit
surfaces; automatic model-generated edits; new production dependencies;
persisted cross-process plans/undo history; broad performance/security claims;
Sprint 42 and v0.7 release execution. Preserve unrelated user changes.

## Acceptance criteria

Separate verified facts, accepted constraints, alternatives, assumptions, and unresolved decisions. Inventory a deterministic oracle for apply, stale rejection, each failure point, rollback, undo, semantic rebuild, cancellation, and concurrency. Identify exact production owners and an achievable first runtime API slice without external dependencies. Mark Sprint 41 active only after this committed-plan task starts. Stop with SPRINT_BLOCKED_MISSING_DATA if an essential oracle is unavailable; do not manufacture source formats.

## Task-specific validation

Non-mutating planner, paired-source, lifecycle, and policy evidence; documentation validation. Use `docs/codex/core/validation.md` as the canonical matrix;
report zero matches separately. Validate this suite explicitly when prompts or
its efficiency records change. Keep large logs under
`local-artifacts/codex-runs/sprint-41/` and return only compact exact evidence.

## Investigation objective

Determine the implementable transaction, authorization, publication, and recovery boundaries.

## Questions to answer

Which service serializes mutation with every publication path? How is an authorization bound to this service lifetime and exact plan? What can filesystem primitives actually guarantee across multiple files, external writers, cancellation, and process interruption? Which complete source set must be rechecked to prevent semantically stale edits outside the touched documents?

## Evidence scope

Current production symbols and consumers only; bound searches before reading files.

## Evidence sources / fixtures

Locate the tracked Sprint 14 paired EDT/Designer corpus through the existing adapter tests and README. Use repository-local temporary copies for mutation evidence later.

## Completion Criteria

Every architecture question has evidence or an explicit blocking/deferred decision; no production mutation is implemented.

## Suggested commit message

```text
Investigate Sprint 41 Safe Edit Transactions
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.

