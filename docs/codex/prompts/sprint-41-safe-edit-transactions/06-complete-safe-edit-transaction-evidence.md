---
prompt_contract: v2
task_kind: architecture
profile: docs/codex/profiles/architecture.md
template: docs/codex/templates/architecture-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Complete Safe Edit Transaction Evidence

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
- `docs/reviews/sprint-41-safe-edit-transactions-design.md` — design decision and risks.
- Task 5 exact committed diff and retained validation summaries, supplied by the dispatcher.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Implement Sprint 41 Safe Edit Transactions` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean task-owned tree. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

## Task

Exact final implementation evidence, consumer audit, and immutable review handoff.

## Scope

### Included

Create `docs/architecture/safe-edit-transactions-evidence.md`; update only relevant Sprint 41 evidence and transaction sections of `docs/Roadmap.md`, `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md`. Reconcile counts from the stable implementation head rather than adding historical totals.

### Excluded

Other refactoring families; metadata/file/path renames; multi-Configuration or
cross-Workspace mutation; remote/Git mutation; new MCP/HTTP/CLI/LSP/IDE edit
surfaces; automatic model-generated edits; new production dependencies;
persisted cross-process plans/undo history; broad performance/security claims;
Sprint 42 and v0.7 release execution. Preserve unrelated user changes.

## Acceptance criteria

Map every invariant to exact committed production and test locations, command statuses/non-zero counts, failure/recovery outcomes, and preserved boundaries. Audit public APIs/consumers, dependencies, Graph/Coverage, cache, protocol/client catalog and sensitive data. Report skips/platform limitations and excluded crash guarantees honestly. Record the exact implementation range and full validation log paths. No production fixes, completion transition, or old-suite deletion belongs here; a defect or missing required result blocks the handoff.

## Task-specific validation

Stable-head test/log reconciliation, API/dependency/Coverage audit, documentation checks. Use `docs/codex/core/validation.md` as the canonical matrix;
report zero matches separately. Validate this suite explicitly when prompts or
its efficiency records change. Keep large logs under
`local-artifacts/codex-runs/sprint-41/` and return only compact exact evidence.

## Suggested commit message

```text
Document Sprint 41 Safe Edit Transaction Evidence
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.

