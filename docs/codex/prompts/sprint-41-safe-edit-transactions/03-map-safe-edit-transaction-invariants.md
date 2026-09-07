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

# Map Safe Edit Transaction Invariants

## Reporting

Communicate in Russian; keep repository artifacts and commit messages in English.
Compose the selected Profile, base Task Template, specialized Template, and their
required Core/Workflow modules; permanent rules remain in those owners.

## Context manifest

### Must read

- `AGENTS.md` — repository boundary and branch/review workflow.
- `docs/Roadmap.md` — sections: Sprint 41 Safe Edit Transactions execution plan and Sprint efficiency contract within that plan.
- `docs/adr/0064-safe-edit-transactions.md` — accepted Decision and Deferred scope.
- `docs/architecture/safe-edit-transactions-invariants.md` — section: Sprint 41 ADR invariant matrix.
- `apps/runtime/src/workspace/mod.rs` — exact affected owners and all publication writers selected by ADR-0064.
- `docs/adr/0063-refactoring-planner.md` — sections: Publication and target identity; Admission bounds; Workspace lifecycle and persistent cache.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Define Sprint 41 Safe Edit Transactions` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean task-owned tree. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

## Task

Complete accepted-ADR production invariant matrix.

## Scope

### Included

Replace provisional rows in `docs/architecture/safe-edit-transactions-invariants.md` with the complete ADR-0064 production matrix; update the Sprint 41 Roadmap matrix evidence. Proposed new production symbols must be explicitly marked planned, with exact paths and signatures/roles. Record documentation and governance requirements separately.

### Excluded

Other refactoring families; metadata/file/path renames; multi-Configuration or
cross-Workspace mutation; remote/Git mutation; new MCP/HTTP/CLI/LSP/IDE edit
surfaces; automatic model-generated edits; new production dependencies;
persisted cross-process plans/undo history; broad performance/security claims;
Sprint 42 and v0.7 release execution. Preserve unrelated user changes.

## Acceptance criteria

For every production invariant give its exact existing/planned owner, production location, operation/retention point it must precede, one negative production-path oracle, and a concrete non-zero focused command. Cover guards before allocation, source read, backup, first write, each later mutation, publication, and recovery. Resolve all provisional rows and cross-layer obligations. Confirm Task 5 scope fits the committed 16-path/5000-line baseline and 12-focused/1-full budget; unexplained growth or missing ownership blocks design review.

## Task-specific validation

Every applicable ADR invariant mapped to location, ordering, negative oracle, and focused command. Use `docs/codex/core/validation.md` as the canonical matrix;
report zero matches separately. Validate this suite explicitly when prompts or
its efficiency records change. Keep large logs under
`local-artifacts/codex-runs/sprint-41/` and return only compact exact evidence.

## Suggested commit message

```text
Map Sprint 41 Safe Edit Transaction Invariants
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.
