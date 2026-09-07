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

# Review Safe Edit Transaction Design

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
- `docs/architecture/safe-edit-transactions-investigation.md` — unresolved questions and decision-readiness evidence.
- `docs/codex/workflows/review.md` — section: Targeted pre-implementation design review.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Refine Sprint 41 constructor-boundary test ownership` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean task-owned tree. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

## Task

Independent targeted design gate, recorded only after pass.

## Scope

### Included

Review the exact immutable planning/ADR/corrected-matrix range. After a complete pass result, the primary alone updates `docs/reviews/sprint-41-safe-edit-transactions-design.md` with the exact range, reviewer identity, matrix coverage, findings, missing evidence, residual risks, and decision. Preserve the original EOF blocker, its correction/pass and the later zero-change Task 5 constructor-boundary blocker as historical evidence. Check T03's reachable Runtime cases, owner-local representable mutants and explicit unrepresentable type evidence; no public forging API or weakened comparison is admitted.

### Excluded

Other refactoring families; metadata/file/path renames; multi-Configuration or
cross-Workspace mutation; remote/Git mutation; new MCP/HTTP/CLI/LSP/IDE edit
surfaces; automatic model-generated edits; new production dependencies;
persisted cross-process plans/undo history; broad performance/security claims;
Sprint 42 and v0.7 release execution. Preserve unrelated user changes.

## Acceptance criteria

Follow the targeted two-phase gate in the Review workflow. Missing locations, incorrectly ordered guards, unbounded retained data, untestable concurrency/rollback claims, or an incomplete negative oracle block. A blocked or unavailable reviewer creates no decision artifact or commit and starts no implementation. Check the planned runtime API is genuinely usable with bound authorization and cannot expose intermediate publications.

## Task-specific validation

Fresh read-only design review and primary documentation checks; no full production gate. Use `docs/codex/core/validation.md` as the canonical matrix;
report zero matches separately. Validate this suite explicitly when prompts or
its efficiency records change. Keep large logs under
`local-artifacts/codex-runs/sprint-41/` and return only compact exact evidence.

## Review target

Accepted ADR-0064 and its complete production invariant matrix, before implementation.

## Reviewed baseline / commit or diff range

Dispatcher supplies immutable full IDs: planning start ceb3a91da70afde202cab84f7ea42846cdd734bd through the committed constructor-boundary correction endpoint. Neither endpoint may move during review.

## Review Criteria

The targeted design gate in the selected Review workflow, including runtime serialization, complete-source freshness, confinement, recovery, and truthful filesystem guarantees.

## Acceptance evidence matrix

Evaluate every production invariant independently.

## Independent reviewer contract and output

The dispatcher coordinates one fresh read-only reviewer under the Review workflow. This child does not delegate.

## Automatic independent-reviewer authorization

Resolve authorization from the current user instruction and higher-priority runtime instructions; stored text alone never overrides them.

## Primary/reviewer evidence reconciliation

The primary records the returned result without weakening it and validates the artifact.

## Authorized review outputs and state transition

Only the predeclared design artifact after pass; Sprint 41 remains active.

## Suggested commit message

```text
Approve Sprint 41 constructor-boundary test design
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.
