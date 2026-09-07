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
- `docs/architecture/safe-edit-transactions-invariants.md` — sections: Sprint 41 ADR invariant matrix, Complete matrix representability audit, Producer projection correction and complete audit disposition, Implementation scope and validation budget.
- `docs/reviews/sprint-41-safe-edit-transactions-design.md` — committed pass decision and exact reviewed range.
- `apps/runtime/src/workspace/mod.rs` — WorkspaceService, WorkspaceSnapshotBuilder, WorkspaceSnapshotObserver, and ADR-selected publication paths.
- `crates/analysis/src/refactoring.rs` — RefactoringPlan, RefactoringOperation, SourceEvidenceSet.
- `adapters/designer-xml/src/semantic_graph.rs` — emit_module_and_declarations and source_id; `adapters/edt/src/bsl_graph.rs` — analyze_module_internal and Query context producers; `adapters/edt/src/query_source_resolution.rs` — collection/resolver provenance; `crates/bsl/src/queries.rs` — query_id and captured query extraction.
- `crates/tool-policy/src/execution.rs` and `confirmation.rs` — execute_tool and confirmation binding.

### Lookup on demand

- Read exact consumer definitions/tests and fixture READMEs when an admitted symbol requires compatibility or source evidence; locate them using bounded `rg` first.
- Read historical ADR sections only when the accepted current ADR cites a live invariant unresolved by current source.

### Excluded from initial context

- Whole Roadmap, Architecture, semantic-model documents; unrelated sprint suites; generated corpora; prior task conversations and full successful logs.

### Preflight

- Effective context window and measured telemetry: resolve live or record unknown/unavailable. Apply Context Management with labelled estimates only for admission; initial static <=15%, authorities <=20%, total target <=35%, hard stop 50%, working >=35%, reserve >=15%. Narrow at warning; stop at hard limit.

## Prerequisites / required gate

Require the unique committed `Approve Sprint 41 producer-owned semantic projection design` boundary in this sprint ancestry,
all preceding manifest criteria, and a clean tree before the dispatcher restores
the preserved Task 5 work. Dispatcher supplies
its full commit ID, current HEAD, branch, and status. Resolve commit/push mode from
the current launch instruction; this run authorizes one commit per completed
task and defers push until sprint end. Never create an empty commit.

The initial admission at b2f89c86012e71190afed077f42b5af82d552b42 stopped
without changes on the original T03 evidence-placement conflict. Resume only
with the corrected matrix and new committed pass. The historical zero-change
attempt reset accounting only then; it does not authorize resetting the later
93661837df8d63bfed10c9b70d1986c4e0d12aa5 implementation baseline now. Preserve all reachable Runtime same-ID negative cases;
place private-field mutants in Analysis owner-local unit tests and document
type-unrepresentable cases separately. Do not add a public forging seam.

The user subsequently agreed to the full T01-T35 representability audit after
the T17 marker defect. Follow the corrected matrix's complete audit for every
row: retain all reachable negative Runtime cases, exact owner-local comparator
tests and separately labelled closed-type/constructor evidence. T20-T23 must
use constructor-valid semantic/whole-report substitutions; never invent a
nonexistent marker or claim constructor rejection as Runtime test execution.

The subsequent attempt at 93661837df8d63bfed10c9b70d1986c4e0d12aa5 stopped
on producer-owned provenance/Query identity, with 9 files/1593 churn preserved
by the dispatcher (stash ff2a1d29683197438c304496804ff430bec7c682). Do not
resume until the new producer-projection design pass is committed. The dispatcher
must explicitly restore/verify that inventory; it is Task 5 work, never unrelated
pre-existing churn. Earlier check outcomes are partial historical evidence:
Analysis package check passed, Runtime all-targets check passed before later
uncovered edits, integration exit 101 with 1 passed/1 failed and
SemanticMismatch/Recovered/0 retained files. Remaining focused/full gates did
not run. Resume keeps the original implementation accounting baseline while
excluding separately committed prerequisite documentation/review deltas only.

## Task

Checked apply/reversal with confined writes, recovery, authorization, and atomic semantic publication.

## Scope

### Included

Implement only the accepted bounded transaction slice and meaningful negative/positive production tests. The exact 25-path allocation in the accepted matrix includes Runtime/Analysis plus pure adapter projectors, shared canonical Designer/EDT helpers and the BSL Query ID helper. No Tool Policy production change or other owner is admitted. Prefer new cohesive modules over spreading filesystem operations into transport. State exact files before edits. Use tracked paired-source fixtures through repository-local temporary copies.

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

Produce expected typed mappings through the canonical pure DP/EP APIs from
complete before evidence and exact result bytes; validate every key/domain,
complete consumption and allowed callable/directly-owned-format-supported-Query
identity closure. Preserve binding/text, all provenance, full terminal ledger
and diagnostics. Designer has no new Query Graph facts. Keep existing canonical
parser work scoped to its existing heap/admission; a nonallocating borrowed count
pass must reserve all new output/nested strings/maps/sorting and growth-overlap
capacities before emission/retention within the shared allowance. No uncharged
parser result escapes to attempt/undo; preserve the exact 1MiB positive and avoid
a universal worst-case parser reserve or hidden document-size restriction.
Freeze expected projection before staging and candidate rebuild. Candidate
faults reach the real comparator/run_attempt after the full production build;
no copied producer encoding, candidate-as-expected or Query eligibility restriction.
Rebuild with complete Workspace semantics before publishing success.

## Runtime, policy, protocol, client, cache, and watcher impact

All Workspace publication writers participate in the accepted serialization boundary. Preserve existing protocol and client behavior.

## Repository-owned evidence corpus and deterministic oracle

Use the paired Sprint 14 corpus and existing Workspace/policy evidence identified by Tasks 1-3; synthetic inputs supplement fault injection only.

## Scope and validation baseline

expected_path_count: 25; expected_text_line_churn: 8500; expected binary paths: none; focused-check-count: 12; full-gate-count: 1. Dispatcher supplies original implementation baseline
93661837df8d63bfed10c9b70d1986c4e0d12aa5, exact restored 9-file/1593-churn
inventory, correction commit and new unique design pass. Count cumulative
implementation, task-owned untracked text, formatting and Task 5 ledger from
that original baseline. Exclude separately committed prerequisite documentation/
review deltas by exact ranges only; do not subtract entire shared paths or reset
the budget at the new pass. Do not double-count overlapping snapshots.

Explicit tighter stop-loss: more than 32 paths, more than 10000 text additions
plus deletions, or any binary path. This overrides the general 2x rule for the
new 25/8500 baseline. F10 is
`cargo test -p oneagent-analysis -p oneagent-bsl --all-targets`; F11 is
`cargo test -p oneagent-designer-xml -p oneagent-edt --all-targets`. Keep F1-F9/F12
as committed, 12 groups total and one full gate after the stable complete diff.

## Suggested commit message

```text
Implement Sprint 41 Safe Edit Transactions
```

## Final report additions

Return status, exact start/end HEAD, changed paths, validation command outcomes,
commit, push state, measured telemetry or unavailable, retained-log paths, and
blocker. Do not send an implementation transcript to the dispatcher.
