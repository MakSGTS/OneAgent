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

# Define Safe Edit Transactions

## Reporting

Communicate in Russian; keep repository artifacts and commit messages in English.
Compose the selected Profile, base Task Template, specialized Template, and their
required Core/Workflow modules; permanent rules remain in those owners.

## Context manifest

### Must read

- `AGENTS.md` — repository boundary and branch/review workflow.
- `docs/Roadmap.md` — sections: Sprint 41 Safe Edit Transactions execution plan and Sprint efficiency contract within that plan.
- `docs/architecture/safe-edit-transactions-investigation.md` — sections answering ownership, authorization, filesystem, recovery, and semantic validation questions.
- `docs/adr/0063-refactoring-planner.md` — sections: Authority, owner, and dependency direction; Publication and target identity; Request, preconditions, and plan identity; Workspace lifecycle and persistent cache; Deferred scope.
- `apps/runtime/src/workspace/mod.rs` — symbols and publication writers identified by the investigation.
- `crates/tool-policy/src/confirmation.rs` and `execution.rs` — confirmation binding and execution authorization path.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Investigate Sprint 41 Safe Edit Transactions` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean task-owned tree. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

## Task

Accepted bounded transaction ADR and compatibility contract.

## Scope

### Included

Create `docs/adr/0064-safe-edit-transactions.md`; modify only the relevant refactoring/transaction sections in `docs/Architecture.md`, `docs/architecture/semantic-model-2.md`, and Sprint 41 Roadmap architecture evidence. Define one local runtime Rust API for checked apply and checked reversal of the existing bsl_callable_rename_v1 family over EDT and Designer XML.

### Excluded

Other refactoring families; metadata/file/path renames; multi-Configuration or
cross-Workspace mutation; remote/Git mutation; new MCP/HTTP/CLI/LSP/IDE edit
surfaces; automatic model-generated edits; new production dependencies;
persisted cross-process plans/undo history; broad performance/security claims;
Sprint 42 and v0.7 release execution. Preserve unrelated user changes.

## Acceptance criteria

Accept exact domain/IO/runtime/policy owners, service-lifetime binding, one-use authorization, plan equivalence, complete-source freshness, operation ordering, bounded retention, write and backup ownership, path/alias confinement, concurrency model, commit point, cleanup, rollback failure precedence, undo freshness, cancellation/shutdown, crash limits, and redaction. Serialize all competing Workspace publication paths and define cache/impact/diagnostic outcomes. Define post-edit success with a complete production rebuild, exact renamed target/calls and unaffected-evidence equivalence; parsing alone is insufficient. State honest multi-file visibility and durability limits and rejected alternatives. Preserve canonical publication identity and immutable old snapshots. No new production dependency or public API removal is authorized. If the investigated slice cannot meet this boundary, report a blocker instead of silently expanding scope.

## Task-specific validation

ADR/authority consistency, affected consumer inventory, Markdown links. Use `docs/codex/core/validation.md` as the canonical matrix;
report zero matches separately. Validate this suite explicitly when prompts or
its efficiency records change. Keep large logs under
`local-artifacts/codex-runs/sprint-41/` and return only compact exact evidence.

## Suggested commit message

```text
Define Sprint 41 Safe Edit Transactions
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.

