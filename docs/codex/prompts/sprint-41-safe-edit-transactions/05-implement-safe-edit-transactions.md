---
prompt_contract: v2
task_kind: implementation
profile: docs/codex/profiles/refactoring-safe-edits-implementation.md
template: docs/codex/templates/refactoring-safe-edits-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Implement Safe Edit Transactions

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
- `docs/reviews/sprint-41-safe-edit-transactions-design.md` — committed pass decision and exact reviewed range.
- `apps/runtime/src/workspace/mod.rs` — WorkspaceService, WorkspaceSnapshotBuilder, WorkspaceSnapshotObserver, and ADR-selected publication paths.
- `crates/analysis/src/refactoring.rs` — RefactoringPlan, RefactoringOperation, SourceEvidenceSet.
- `crates/tool-policy/src/execution.rs` and `confirmation.rs` — execute_tool and confirmation binding.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Approve Sprint 41 Safe Edit Transaction Design` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean task-owned tree. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

## Task

Checked apply/reversal with confined writes, recovery, authorization, and atomic semantic publication.

## Scope

### Included

Implement only the accepted bounded transaction slice and meaningful negative/positive production tests. Expected areas are `apps/runtime/src/workspace/`, Runtime exports as needed, and `apps/runtime/tests/`; pure Analysis or Tool Policy changes require the ADR's exact owner/consumer justification. Prefer new cohesive modules over spreading filesystem operations into transport. State exact files before edits. Use tracked paired-source fixtures through repository-local temporary copies.

### Excluded

Other refactoring families; metadata/file/path renames; multi-Configuration or
cross-Workspace mutation; remote/Git mutation; new MCP/HTTP/CLI/LSP/IDE edit
surfaces; automatic model-generated edits; new production dependencies;
persisted cross-process plans/undo history; broad performance/security claims;
Sprint 42 and v0.7 release execution. Preserve unrelated user changes.

## Acceptance criteria

Every committed matrix row has its guard at the accepted production location and a passing oracle. Apply consumes a fresh complete planner result and exact one-use permission; stale publication, untouched-source semantic changes, token/range mismatch, aliases/symlinks, conflicting apply/undo, unauthorized calls and all accepted bounds fail closed before side effects. Preserve preimages before writes, inject each accepted write/restore/validation/cleanup failure, distinguish recovery outcomes, and never overwrite unrelated external changes during recovery. Prove full production rebuild target/calls/diagnostic invariants, unchanged unrelated files, old Arc immutability, successful undo equivalence, and serialized watcher/cache/publication behavior. Audit every public consumer and preserve planner, eight-tool MCP catalog, cache identity, Coverage and Graph contracts. Run focused checks while changing, then the canonical full gate once on the stable complete diff. Stop at the committed scope limit before further work.

## Task-specific validation

12 focused checks defined by Task 3; one stable canonical full workspace gate. Use `docs/codex/core/validation.md` as the canonical matrix;
report zero matches separately. Validate this suite explicitly when prompts or
its efficiency records change. Keep large logs under
`local-artifacts/codex-runs/sprint-41/` and return only compact exact evidence.

## Canonical semantic, source, repository, and authorization boundaries

ADR-0063 owners and ADR-0064 transaction owners; no plan is authorization.

## Supported refactoring family and production source entry point

bsl_callable_rename_v1 in paired EDT/Designer XML through the accepted runtime Rust API; no wire/UI endpoint.

## Target, snapshot, source-version, and precondition contract

Use the accepted service-lifetime, complete-source and exact-plan checks before mutation.

## Plan and operation identity, vocabulary, ordering, and dependencies

Reuse canonical planner identities and ordering without a competing publication counter.

## Duplicate, overlap, conflict, stale-input, and incompatibility behavior

Enforce the ADR matrix through production entry points, including racing publication and unrelated semantic source changes.

## Bounds, completeness, preview, failures, and sensitive-data policy

Check bounds before retaining input/preimages/results; keep errors and Debug output redacted.

## Transaction, filesystem confinement, atomicity, rollback, and reversibility

Implement the exact accepted filesystem/concurrency model, failure precedence, retained recovery evidence, and checked undo; make no stronger crash or multi-file guarantee.

## Post-edit production rebuild and semantic validation

Rebuild with accepted source adapters and complete Workspace semantics before publishing success.

## Runtime, policy, protocol, client, cache, and watcher impact

All Workspace publication writers participate in the accepted serialization boundary. Preserve existing protocol and client behavior.

## Repository-owned evidence corpus and deterministic oracle

Use the paired Sprint 14 corpus and existing Workspace/policy evidence identified by Tasks 1-3; synthetic inputs supplement fault injection only.

## Scope and validation baseline

expected_path_count: 16; expected_text_line_churn: 5000; expected binary paths: none; focused-check-count: 12; full-gate-count: 1. Dispatcher supplies exact task-start commit, initial untracked inventory, matrix commit, and design pass commit. Apply the sequential workflow's scope accounting and stop-loss.

## Suggested commit message

```text
Implement Sprint 41 Safe Edit Transactions
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.
