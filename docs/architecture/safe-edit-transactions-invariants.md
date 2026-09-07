# Safe Edit Transaction Invariants

Planning baseline: Sprint 41 from `ceb3a91da70afde202cab84f7ea42846cdd734bd`.
Status: provisional; no design decision or production conformance is claimed.
Task 2 owns ADR-0064; Task 3 must replace these questions with the complete
accepted production mapping before Task 4. Exact new symbols cannot be accepted
until their owner and operation order are designed from current code.

## Sprint 41 ADR invariant matrix

| Obligation | Confirmed existing boundary | Production location/order still to resolve | Required negative oracle |
|---|---|---|---|
| One canonical publication and service lifetime | Analysis publication identity; Runtime WorkspaceService | Exact transaction admission before retaining a plan or permission; every publication writer | Equal numeric publication from another service or stale publication rejects before mutation |
| Complete fresh semantic/source preconditions | RefactoringPlan and SourceEvidenceSet; immutable WorkspaceSnapshot | Complete current-source recheck before first mutation, including untouched source relevant to semantics | Untouched source adds a conflicting call/target after preview; no write |
| Exact bound authorization | Tool Policy confirmation and execute_tool | Exact actor, request, service lifetime, plan and one-use permission before mutation | Missing, changed, reused or cross-service permission cannot reach writer |
| Confined bounded IO | Existing confined source paths and production source capture | Every read, preimage retention, staging, write, restore and cleanup; alias/symlink/size rules | Traversal, symlink/alias substitution or over-bound source touches no unauthorized path |
| Checked ordered complete writes | Canonical planner operation order and expected bytes | Preimage preservation, per-operation recheck, commit and recovery points | Inject each accepted failure and external edit without false success or lost unrelated data |
| Reversal and recovery | New contract required by Sprint 41 | Undo binding, current-state recheck, rollback-failure precedence and retained recovery evidence | Stale undo, rollback failure or cancellation never reports successful complete reversal |
| Complete semantic validation | WorkspaceSnapshotBuilder and production EDT/Designer builders | Candidate validation before publication/cache success; all watcher/rebuild writers coordinated | Invalid renamed target/calls or failed rebuild publishes no successful candidate |
| Deterministic bounded results and compatibility | Existing planner, Graph/Coverage, cache and runtime observers | Bounds before retention; redacted errors/results; no competing counter or wire changes | Over-limit, reordered, replayed and fault cases preserve existing consumers and emit no source secrets |

Each final row must add exact accepted ADR clause, production owner/path/symbol,
the guarded operation or retention point, one executable negative production
oracle, and a concrete non-zero focused validation command. Split rows whenever
one broad row would hide multiple locations or different ordering obligations.
A row labelled pending or unresolved blocks the targeted design review.

## Separate non-production evidence

Documentation accuracy, exact-head test enumeration, dependency/API/Coverage
audits, reviewer independence, scope accounting and prompt retirement belong in
the evidence/review artifacts. Deferred UI, wire endpoints, new refactoring
families, persistent undo, release work and broad durability claims are not
production invariants for this bounded sprint.

## Validation budget

Task 5 baseline: 16 unique paths, 5000 textual additions plus deletions, no
binary paths, 12 concrete focused checks selected by Task 3, and one canonical
full workspace gate on its stable complete diff. Report actual scope against
the exact task-start commit. The independent reviewer and primary integration
review perform separate required validation.

